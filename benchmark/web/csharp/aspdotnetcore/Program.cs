using System.Net;
using System.Text.Json;
using System.Text.Json.Serialization;

var builder = WebApplication.CreateBuilder(args);
builder.WebHost.ConfigureKestrel(o => o.AddServerHeader = false);
builder.Logging.ClearProviders();

builder.Services.AddHttpClient<IBenchmarkService, BenchmarkService>(client =>
{
	client.BaseAddress = new Uri("http://web-data-source/");
	client.DefaultRequestHeaders.ConnectionClose = false;
}).ConfigurePrimaryHttpMessageHandler(() => new SocketsHttpHandler
{
	UseProxy = false,
	UseCookies = false,
	AutomaticDecompression = DecompressionMethods.None
	//AutomaticDecompression = DecompressionMethods.Brotli | DecompressionMethods.GZip
});

var app = builder.Build();

app.MapGet("/api/v1/periodic-table/element", async (IBenchmarkService benchService, string symbol) =>
{
	var element = await benchService.GetElementAsync(symbol);
#if NET8_0_OR_GREATER
	return TypedResults.Json(element, BenchmarkJsonContext.Default.ElementData);
#else
	return Results.Json(element, BenchmarkJsonContext.Default.Options);
#endif
});

app.MapGet("/api/v1/periodic-table/shells", async (IBenchmarkService benchService, string symbol) =>
{
	var shells = await benchService.GetShellsAsync(symbol);
#if NET8_0_OR_GREATER
	return TypedResults.Json(shells, BenchmarkJsonContext.Default.ShellData);
#else
	return Results.Json(shells, BenchmarkJsonContext.Default.Options);
#endif
});

app.Run();

public interface IBenchmarkService
{
	Task<ElementData> GetElementAsync(string symbol);
	Task<ShellData> GetShellsAsync(string symbol);
}

public sealed class BenchmarkService : IBenchmarkService
{
	private readonly HttpClient _client;

	public BenchmarkService(HttpClient httpClient)
	{
		_client = httpClient;
	}

#if NET8_0_OR_GREATER

	public async Task<ElementData> GetElementAsync(string symbol)
	{
		var json = await _client.GetFromJsonAsync("element.json", BenchmarkJsonContext.Default.JsonDocument).ConfigureAwait(false);
		var root = json!.RootElement;
		var entry = root.GetProperty(symbol);
		var entryData = entry.Deserialize(BenchmarkJsonContext.Default.ElementData)!;
		return entryData;
	}

	public async Task<ShellData> GetShellsAsync(string symbol)
	{
		var json = await _client.GetFromJsonAsync("shells.json", BenchmarkJsonContext.Default.JsonDocument).ConfigureAwait(false);
		var root = json!.RootElement;
		var entry = root.GetProperty(symbol);
		var shells = entry.Deserialize(BenchmarkJsonContext.Default.Int32Array)!;
		return new ShellData(shells);
	}

#else

	public async Task<ElementData> GetElementAsync(string symbol)
	{
		var json = await _client.GetFromJsonAsync("element.json", BenchmarkJsonContext.Default.DictionaryStringElementData).ConfigureAwait(false);
		var entry = json![symbol]!;
		return entry;
	}

	public async Task<ShellData> GetShellsAsync(string symbol)
	{
		var json = await _client.GetFromJsonAsync("shells.json", BenchmarkJsonContext.Default.DictionaryStringInt32Array).ConfigureAwait(false);
		var entry = json![symbol]!;
		return new ShellData(entry);
	}

#endif
}

public sealed class ElementData
{
	public string? Name { get; set; }
	public int Number { get; set; }
	public int Group { get; set; }
}

public sealed class ShellData
{
	public int[] Shells { get; set; }

	public ShellData(int[] shells)
	{
		Shells = shells;
	}
}

#if NET8_0_OR_GREATER
[JsonSourceGenerationOptions(JsonSerializerDefaults.Web)]
#else
[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase)]
#endif
[JsonSerializable(typeof(Dictionary<string, ElementData>))]
[JsonSerializable(typeof(Dictionary<string, int[]>))]
[JsonSerializable(typeof(ShellData))]
[JsonSerializable(typeof(JsonDocument))]
public partial class BenchmarkJsonContext : JsonSerializerContext { }
