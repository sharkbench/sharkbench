using System.Buffers;
using System.Buffers.Text;
using System.Net;
using System.Text;
using System.Text.Json;
using Microsoft.AspNetCore.Hosting.Server;
using Microsoft.AspNetCore.Http.Features;
using Microsoft.AspNetCore.Server.Kestrel.Core;
using Microsoft.AspNetCore.Server.Kestrel.Transport.Sockets;
using Microsoft.Extensions.Logging.Abstractions;
using Microsoft.Extensions.Options;

// 1. Direct loopback HttpClient (zero DNS/proxy/cookie overhead)
var socketsHandler = new SocketsHttpHandler
{
    UseProxy = false,
    UseCookies = false,
    AllowAutoRedirect = false,
    AutomaticDecompression = DecompressionMethods.None,
    MaxConnectionsPerServer = 64,
    PooledConnectionIdleTimeout = TimeSpan.FromMinutes(5),
    ConnectTimeout = TimeSpan.FromSeconds(5)
};

using var httpClient = new HttpClient(socketsHandler)
{
    // host.docker.internal resolves to the host machine from inside a Docker container
    BaseAddress = new Uri("http://host.docker.internal:5002"),
    DefaultRequestVersion = HttpVersion.Version11,
    DefaultVersionPolicy = HttpVersionPolicy.RequestVersionExact
};

// 2. Raw Kestrel server with inline scheduling (avoids ThreadPool dispatch & CFS throttling)
var socketOptions = Options.Create(new SocketTransportOptions
{
    IOQueueCount = 1,
    NoDelay = true,
    UnsafePreferInlineScheduling = true
});
var transportFactory = new SocketTransportFactory(socketOptions, NullLoggerFactory.Instance);

var kestrelOptions = Options.Create(new KestrelServerOptions
{
    AddServerHeader = false
});
kestrelOptions.Value.ListenAnyIP(5001, listenOptions =>
{
    listenOptions.Protocols = HttpProtocols.Http1;
});

using var server = new KestrelServer(kestrelOptions, transportFactory, NullLoggerFactory.Instance);
var app = new PeriodicTableApp(httpClient);

using var cts = new CancellationTokenSource();
Console.CancelKeyPress += (_, e) => { e.Cancel = true; cts.Cancel(); };
AppDomain.CurrentDomain.ProcessExit += (_, _) => cts.Cancel();

await server.StartAsync(app, cts.Token);
await Task.Delay(Timeout.Infinite, cts.Token).ConfigureAwait(ConfigureAwaitOptions.SuppressThrowing);
await server.StopAsync(CancellationToken.None);

// 3. Raw Kestrel IHttpApplication<TContext> implementation
sealed class PeriodicTableApp(HttpClient client) : IHttpApplication<PeriodicTableApp.Context>
{
    private const string ElementPath = "/api/v1/periodic-table/element";
    private const string ShellsPath = "/api/v1/periodic-table/shells";
    private static readonly Uri ElementUpstreamUri = new("/element.json", UriKind.Relative);
    private static readonly Uri ShellsUpstreamUri = new("/shells.json", UriKind.Relative);

    public struct Context
    {
        public IHttpRequestFeature Request;
        public IHttpResponseFeature Response;
        public IHttpResponseBodyFeature ResponseBody;
    }

    public Context CreateContext(IFeatureCollection features) => new()
    {
        Request = features.Get<IHttpRequestFeature>()!,
        Response = features.Get<IHttpResponseFeature>()!,
        ResponseBody = features.Get<IHttpResponseBodyFeature>()!
    };

    public void DisposeContext(Context context, Exception? exception) { }

    public Task ProcessRequestAsync(Context ctx)
    {
        var path = ctx.Request.Path.AsSpan();
        var symbolSpan = ExtractSymbol(ctx.Request.QueryString.AsSpan());

        if (symbolSpan.IsEmpty)
        {
            ctx.Response.StatusCode = 400;
            return Task.CompletedTask;
        }

        Span<byte> utf8Buf = stackalloc byte[8];
        int utf8Len = Encoding.UTF8.GetBytes(symbolSpan, utf8Buf);
        ulong packedSymbol = 0;
        for (int i = 0; i < utf8Len && i < 8; i++)
        {
            packedSymbol |= (ulong)utf8Buf[i] << (i * 8);
        }

        if (path.SequenceEqual(ElementPath.AsSpan()))
        {
            return HandleElementAsync(ctx, packedSymbol, utf8Len);
        }

        if (path.SequenceEqual(ShellsPath.AsSpan()))
        {
            return HandleShellsAsync(ctx, packedSymbol, utf8Len);
        }

        ctx.Response.StatusCode = 404;
        return Task.CompletedTask;
    }

    private static ReadOnlySpan<char> ExtractSymbol(ReadOnlySpan<char> queryString)
    {
        if (queryString.IsEmpty) return default;
        var q = queryString[0] == '?' ? queryString[1..] : queryString;

        while (!q.IsEmpty)
        {
            var ampIdx = q.IndexOf('&');
            var pair = ampIdx >= 0 ? q[..ampIdx] : q;
            if (pair.StartsWith("symbol=".AsSpan(), StringComparison.Ordinal))
            {
                return pair[7..];
            }
            if (ampIdx < 0) break;
            q = q[(ampIdx + 1)..];
        }
        return default;
    }

    private async Task HandleElementAsync(Context ctx, ulong packedSymbol, int symbolLen)
    {
        var (buffer, length) = await FetchUpstreamBytesAsync(client, ElementUpstreamUri).ConfigureAwait(false);
        try
        {
            Span<byte> symbolUtf8 = stackalloc byte[8];
            for (int i = 0; i < symbolLen && i < 8; i++)
                symbolUtf8[i] = (byte)(packedSymbol >> (i * 8));

            if (!TryWriteElementResponse(buffer.AsSpan(0, length), symbolUtf8[..symbolLen], ctx))
            {
                ctx.Response.StatusCode = 404;
            }
        }
        finally
        {
            ArrayPool<byte>.Shared.Return(buffer);
        }
    }

    private async Task HandleShellsAsync(Context ctx, ulong packedSymbol, int symbolLen)
    {
        var (buffer, length) = await FetchUpstreamBytesAsync(client, ShellsUpstreamUri).ConfigureAwait(false);
        try
        {
            Span<byte> symbolUtf8 = stackalloc byte[8];
            for (int i = 0; i < symbolLen && i < 8; i++)
                symbolUtf8[i] = (byte)(packedSymbol >> (i * 8));

            if (!TryWriteShellsResponse(buffer.AsSpan(0, length), symbolUtf8[..symbolLen], ctx))
            {
                ctx.Response.StatusCode = 404;
            }
        }
        finally
        {
            ArrayPool<byte>.Shared.Return(buffer);
        }
    }

    private static async Task<(byte[] Buffer, int Length)> FetchUpstreamBytesAsync(HttpClient client, Uri uri)
    {
        using var response = await client.GetAsync(uri, HttpCompletionOption.ResponseHeadersRead).ConfigureAwait(false);
        int initialSize = (int)(response.Content.Headers.ContentLength ?? 16384);
        byte[] buffer = ArrayPool<byte>.Shared.Rent(initialSize);
        int offset = 0;

        await using var stream = await response.Content.ReadAsStreamAsync().ConfigureAwait(false);
        while (true)
        {
            if (offset == buffer.Length)
            {
                byte[] larger = ArrayPool<byte>.Shared.Rent(buffer.Length * 2);
                Buffer.BlockCopy(buffer, 0, larger, 0, offset);
                ArrayPool<byte>.Shared.Return(buffer);
                buffer = larger;
            }

            int read = await stream.ReadAsync(buffer.AsMemory(offset)).ConfigureAwait(false);
            if (read == 0) break;
            offset += read;
        }

        return (buffer, offset);
    }

    private static bool TryWriteElementResponse(ReadOnlySpan<byte> json, ReadOnlySpan<byte> targetSymbol, Context ctx)
    {
        var reader = new Utf8JsonReader(json);
        if (!reader.Read() || reader.TokenType != JsonTokenType.StartObject) return false;

        while (reader.Read() && reader.TokenType != JsonTokenType.EndObject)
        {
            if (reader.TokenType != JsonTokenType.PropertyName) continue;

            if (!reader.ValueTextEquals(targetSymbol))
            {
                reader.Read();
                reader.Skip();
                continue;
            }

            if (!reader.Read() || reader.TokenType != JsonTokenType.StartObject) return false;

            ReadOnlySpan<byte> nameUtf8 = default;
            int number = 0;
            int group = 0;

            while (reader.Read() && reader.TokenType != JsonTokenType.EndObject)
            {
                if (reader.TokenType != JsonTokenType.PropertyName) continue;

                if (reader.ValueTextEquals("name"u8))
                {
                    reader.Read();
                    nameUtf8 = reader.ValueSpan;
                }
                else if (reader.ValueTextEquals("number"u8))
                {
                    reader.Read();
                    number = reader.GetInt32();
                }
                else if (reader.ValueTextEquals("group"u8))
                {
                    reader.Read();
                    group = reader.GetInt32();
                }
                else
                {
                    reader.Read();
                    reader.Skip();
                }
            }

            ctx.Response.StatusCode = 200;
            ctx.Response.Headers.ContentType = "application/json";

            Span<byte> outSpan = ctx.ResponseBody.Writer.GetSpan(128 + nameUtf8.Length);
            int pos = 0;
            "{\"name\":\""u8.CopyTo(outSpan[pos..]); pos += 9;
            nameUtf8.CopyTo(outSpan[pos..]); pos += nameUtf8.Length;
            "\",\"number\":"u8.CopyTo(outSpan[pos..]); pos += 11;
            Utf8Formatter.TryFormat(number, outSpan[pos..], out int numWritten); pos += numWritten;
            ",\"group\":"u8.CopyTo(outSpan[pos..]); pos += 9;
            Utf8Formatter.TryFormat(group, outSpan[pos..], out int grpWritten); pos += grpWritten;
            outSpan[pos++] = (byte)'}';

            ctx.ResponseBody.Writer.Advance(pos);
            return true;
        }

        return false;
    }

    private static bool TryWriteShellsResponse(ReadOnlySpan<byte> json, ReadOnlySpan<byte> targetSymbol, Context ctx)
    {
        var reader = new Utf8JsonReader(json);
        if (!reader.Read() || reader.TokenType != JsonTokenType.StartObject) return false;
        while (reader.Read() && reader.TokenType != JsonTokenType.EndObject)
        {
            if (reader.TokenType != JsonTokenType.PropertyName) continue;
            if (!reader.ValueTextEquals(targetSymbol))
            {
                reader.Read();
                reader.Skip();
                continue;
            }
            if (!reader.Read() || reader.TokenType != JsonTokenType.StartArray) return false;
            ctx.Response.StatusCode = 200;
            ctx.Response.Headers.ContentType = "application/json";
            Span<byte> outSpan = ctx.ResponseBody.Writer.GetSpan(128);
            int pos = 0;
            "{\"shells\":["u8.CopyTo(outSpan[pos..]); pos += 11;
            bool first = true;
            while (reader.Read() && reader.TokenType != JsonTokenType.EndArray)
            {
                if (!first)
                {
                    outSpan[pos++] = (byte)',';
                }
                first = false;
                Utf8Formatter.TryFormat(reader.GetInt32(), outSpan[pos..], out int written);
                pos += written;
            }
            outSpan[pos++] = (byte)']';
            outSpan[pos++] = (byte)'}';
            ctx.ResponseBody.Writer.Advance(pos);
            return true;
        }
        return false;
    }
}