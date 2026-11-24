using System.Buffers;
using System.IO.Pipelines;
using System.Net;
using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.AspNetCore.Hosting.Server;
using Microsoft.AspNetCore.Hosting.Server.Features;
using Microsoft.AspNetCore.Http.Features;
using Microsoft.AspNetCore.Server.Kestrel.Core;
using Microsoft.AspNetCore.Server.Kestrel.Transport.Sockets;
using Microsoft.Extensions.Logging.Abstractions;
using Microsoft.Extensions.Options;

// 1. Configure high-throughput loopback HttpClient (no proxy, no cookies, pooled HTTP/1.1)
var socketsHandler = new SocketsHttpHandler
{
    UseProxy = false,
    UseCookies = false,
    AllowAutoRedirect = false,
    MaxConnectionsPerServer = 256,
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

// 2. Configure raw Kestrel server directly (no MVC, no Routing, no Middleware pipeline)
var socketOptions = Options.Create(new SocketTransportOptions
{
    IOQueueCount = 1, // Optimal for 1-core Docker CPU cap
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

    public async Task ProcessRequestAsync(Context ctx)
    {
        var path = ctx.Request.Path.AsSpan();
        var symbolSpan = ExtractSymbol(ctx.Request.QueryString.AsSpan());

        if (symbolSpan.IsEmpty)
        {
            ctx.Response.StatusCode = 400;
            return;
        }

        var symbol = symbolSpan.ToString();

        if (path.SequenceEqual(ElementPath.AsSpan()))
        {
            await HandleElementAsync(ctx, symbol);
        }
        else if (path.SequenceEqual(ShellsPath.AsSpan()))
        {
            await HandleShellsAsync(ctx, symbol);
        }
        else
        {
            ctx.Response.StatusCode = 404;
        }
    }

    private static ReadOnlySpan<char> ExtractSymbol(ReadOnlySpan<char> queryString)
    {
        // Expects "?symbol=He" or "?foo=1&symbol=He"
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

    private async Task HandleElementAsync(Context ctx, string symbol)
    {
        using var response = await client.GetAsync(ElementUpstreamUri, HttpCompletionOption.ResponseHeadersRead);
        if (!response.IsSuccessStatusCode)
        {
            ctx.Response.StatusCode = 502;
            return;
        }
        await using var stream = await response.Content.ReadAsStreamAsync();

        // element.json is a JSON object keyed by symbol, e.g. {"He": {"name":"Helium","number":2,"group":18}, ...}
        // (see benchmark/web/csharp/aspdotnetcore and src/benchmark/web/data/static/element.json)
        var elements = await JsonSerializer.DeserializeAsync(stream, AppJsonContext.Default.DictionaryStringUpstreamElement);
        if (elements is null || !elements.TryGetValue(symbol, out var found))
        {
            ctx.Response.StatusCode = 404;
            return;
        }

        ctx.Response.StatusCode = 200;
        ctx.Response.Headers.ContentType = "application/json";
        await JsonSerializer.SerializeAsync(
            ctx.ResponseBody.Stream,
            new ElementResponse(found.Name, found.Number, found.Group),
            AppJsonContext.Default.ElementResponse);
    }

    private async Task HandleShellsAsync(Context ctx, string symbol)
    {
        using var response = await client.GetAsync(ShellsUpstreamUri, HttpCompletionOption.ResponseHeadersRead);
        if (!response.IsSuccessStatusCode)
        {
            ctx.Response.StatusCode = 502;
            return;
        }
        await using var stream = await response.Content.ReadAsStreamAsync();

        // shells.json is a JSON object keyed by symbol, e.g. {"He": [2], ...}
        // (see benchmark/web/csharp/aspdotnetcore and src/benchmark/web/data/static/shells.json)
        var shellsMap = await JsonSerializer.DeserializeAsync(stream, AppJsonContext.Default.DictionaryStringInt32Array);
        if (shellsMap is null || !shellsMap.TryGetValue(symbol, out var shells))
        {
            ctx.Response.StatusCode = 404;
            return;
        }

        ctx.Response.StatusCode = 200;
        ctx.Response.Headers.ContentType = "application/json";
        await JsonSerializer.SerializeAsync(
            ctx.ResponseBody.Stream,
            new ShellsResponse(shells),
            AppJsonContext.Default.ShellsResponse);
    }
}
// 4. Source-Generated JSON Models (Zero Reflection)
public sealed record UpstreamElement(
    [property: JsonPropertyName("name")] string Name,
    [property: JsonPropertyName("number")] int Number,
    [property: JsonPropertyName("group")] int Group);

public sealed record ElementResponse(
    [property: JsonPropertyName("name")] string Name,
    [property: JsonPropertyName("number")] int Number,
    [property: JsonPropertyName("group")] int Group);

public sealed record ShellsResponse(
    [property: JsonPropertyName("shells")] int[] Shells);

[JsonSourceGenerationOptions(GenerationMode = JsonSourceGenerationMode.Default)]
[JsonSerializable(typeof(Dictionary<string, UpstreamElement>))]
[JsonSerializable(typeof(Dictionary<string, int[]>))]
[JsonSerializable(typeof(ElementResponse))]
[JsonSerializable(typeof(ShellsResponse))]
internal partial class AppJsonContext : JsonSerializerContext;