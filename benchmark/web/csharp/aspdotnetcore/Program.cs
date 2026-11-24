using System.Buffers;
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
    MaxConnectionsPerServer = 256,
    PooledConnectionIdleTimeout = TimeSpan.FromMinutes(5)
};

using var httpClient = new HttpClient(socketsHandler)
{
    // Note: Sharkbench runs with --network=host so 127.0.0.1:5002 is direct loopback.
    // Change back to host.docker.internal:5002 only if testing without --network=host.
    BaseAddress = new Uri("http://127.0.0"),
    DefaultRequestVersion = HttpVersion.Version11,
    DefaultVersionPolicy = HttpVersionPolicy.RequestVersionExact
};

// 2. Raw Kestrel server tuned for 1-core Docker CPU cap
var socketOptions = Options.Create(new SocketTransportOptions
{
    IOQueueCount = 1,
    NoDelay = true
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

sealed class PeriodicTableApp(HttpClient client) : IHttpApplication<PeriodicTableApp.Context>
{
    private const string ElementPath = "/api/v1/periodic-table/element";
    private const string ShellsPath = "/api/v1/periodic-table/shells";
    private static readonly Uri ElementUri = new("element.json", UriKind.Relative);
    private static readonly Uri ShellsUri = new("shells.json", UriKind.Relative);

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

    public async Task ProcessRequestAsync(Context ctx)
    {
        var path = ctx.Request.Path.AsSpan();
        var symbol = ExtractSymbol(ctx.Request.QueryString.AsSpan());

        if (symbol.IsEmpty)
        {
            ctx.Response.StatusCode = 400;
            return;
        }

        if (path.SequenceEqual(ElementPath.AsSpan()))
        {
            await HandleElementAsync(ctx, symbol).ConfigureAwait(false);
        }
        else if (path.SequenceEqual(ShellsPath.AsSpan()))
        {
            await HandleShellsAsync(ctx, symbol).ConfigureAwait(false);
        }
        else
        {
            ctx.Response.StatusCode = 404;
        }
    }

    private static ReadOnlySpan<char> ExtractSymbol(ReadOnlySpan<char> queryString)
    {
        if (queryString.IsEmpty) return default;
        var q = queryString == '?' ? queryString[1..] : queryString;

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

    private static async Task<(byte[] Buffer, int Length)> FetchUpstreamBytesAsync(HttpClient client, Uri uri)
    {
        using var response = await client.GetAsync(uri, HttpCompletionOption.ResponseHeadersRead).ConfigureAwait(false);
        int initialSize = (int)(response.Content.Headers.ContentLength ?? 32768);
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

    private async Task HandleElementAsync(Context ctx, ReadOnlySpan<char> symbol)
    {
        Span<byte> symbolUtf8 = stackalloc byte;
        int symbolLen = Encoding.UTF8.GetBytes(symbol, symbolUtf8);
        var target = symbolUtf8[..symbolLen];

        var (buffer, length) = await FetchUpstreamBytesAsync(client, ElementUri).ConfigureAwait(false);
        try
        {
            if (!TryWriteElementResponse(buffer.AsSpan(0, length), target, ctx))
            {
                ctx.Response.StatusCode = 404;
                return;
            }

            await ctx.ResponseBody.Writer.FlushAsync().ConfigureAwait(false);
        }
        finally
        {
            ArrayPool<byte>.Shared.Return(buffer);
        }
    }

    private async Task HandleShellsAsync(Context ctx, ReadOnlySpan<char> symbol)
    {
        Span<byte> symbolUtf8 = stackalloc byte;
        int symbolLen = Encoding.UTF8.GetBytes(symbol, symbolUtf8);
        var target = symbolUtf8[..symbolLen];

        var (buffer, length) = await FetchUpstreamBytesAsync(client, ShellsUri).ConfigureAwait(false);
        try
        {
            if (!TryWriteShellsResponse(buffer.AsSpan(0, length), target, ctx))
            {
                ctx.Response.StatusCode = 404;
                return;
            }

            await ctx.ResponseBody.Writer.FlushAsync().ConfigureAwait(false);
        }
        finally
        {
            ArrayPool<byte>.Shared.Return(buffer);
        }
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
                reader.Skip(); // Zero-allocation jump over non-matching elements
                continue;
            }

            // Found matching symbol! Read its object: {"name":"Helium","number":2,"group":18,...}
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
                    nameUtf8 = reader.ValueSpan; // Raw UTF-8 slice from pooled buffer (zero string alloc!)
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
            using var writer = new Utf8JsonWriter(ctx.ResponseBody.Writer);
            writer.WriteStartObject();
            writer.WriteString("name"u8, nameUtf8);
            writer.WriteNumber("number"u8, number);
            writer.WriteNumber("group"u8, group);
            writer.WriteEndObject();
            writer.Flush();
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
                reader.Skip(); // Zero-allocation jump over non-matching elements
                continue;
            }
            if (!reader.Read() || reader.TokenType != JsonTokenType.StartArray) return false;
            ctx.Response.StatusCode = 200;
            ctx.Response.Headers.ContentType = "application/json";
            using var writer = new Utf8JsonWriter(ctx.ResponseBody.Writer);
            writer.WriteStartObject();
            writer.WritePropertyName("shells"u8);
            writer.WriteStartArray();
            while (reader.Read() && reader.TokenType != JsonTokenType.EndArray)
            {
                writer.WriteNumberValue(reader.GetInt32());
            }
            writer.WriteEndArray();
            writer.WriteEndObject();
            writer.Flush();
            return true;
        }
        return false;
    }
}