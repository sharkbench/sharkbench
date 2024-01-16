using Microsoft.AspNetCore.Mvc;
using System.Net.Http;
using System.Threading.Tasks;
using Newtonsoft.Json.Linq;

namespace web;

[ApiController]
public class Controller : ControllerBase
{
    private readonly HttpClient _httpClient;

    public Controller()
    {
        _httpClient = new HttpClient();
    }

    [HttpGet("/api/v1/periodic-table/element")]
    public async Task<IActionResult> GetElement([FromQuery] string symbol)
    {
        var response = await _httpClient.GetAsync("http://web-data-source/element.json");
        var content = await response.Content.ReadAsStringAsync();
        var json = JObject.Parse(content);
        var entry = json[symbol]!;

        var elementData = new ElementData
        {
            Name = entry["name"]?.ToString(),
            Number = (int)entry["number"],
            Group = (int)entry["group"],
        };

        return Ok(elementData);
    }

    [HttpGet("/api/v1/periodic-table/shells")]
    public async Task<IActionResult> GetShells([FromQuery] string symbol)
    {
        var response = await _httpClient.GetAsync("http://web-data-source/shells.json");
        var content = await response.Content.ReadAsStringAsync();
        var json = JObject.Parse(content);

        var shellData = new ShellData
        {
            Shells = json[symbol]!.ToObject<int[]>(),
        };

        return Ok(shellData);
    }
}

public class ElementData
{
    public string Name { get; set; }
    public int Number { get; set; }
    public int Group { get; set; }
}

public class ShellData
{
    public int[] Shells { get; set; }
}
