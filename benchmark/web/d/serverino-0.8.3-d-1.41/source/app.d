import std.datetime;
import djson;
import serverino;
import requests : req = Request;

mixin ServerinoMain;

// With a single CPU, parallel GC marking threads only compete with the worker
extern(C) __gshared string[] rt_options = ["gcopt=parallel:0"];

__gshared req client = req();

@onServerInit ServerinoConfig configure()
{
	return ServerinoConfig
		.create()
        .setHttpTimeout(15.seconds)
        .enableKeepAlive(100.seconds)
   		.addListener("0.0.0.0", 3000)
        .setDaemonInstances(1)
        .setWorkers(1)
        .enableWorkerBacklog(8);
}

// Single pass over the document: stops as soon as the member named `symbol` has been read
string findEntry(string json, string symbol) {
    string entry;
    json.walkJSON!("$.*", (JValue v, const(PathItem)[] path) {
        if (path[0].key != symbol) return WalkControl.next;
        entry = v.toJSON();
        return WalkControl.stop;
    });
    if (entry is null) throw new Exception("Unknown symbol: " ~ symbol);
    return entry;
}

@endpoint
@route!("/api/v1/periodic-table/element") void elementHandler(Request req, Output output) {
    auto symbol = req.get.read("symbol");
    auto rs = client.get("http://web-data-source/element.json");
    auto entry = findEntry((rs.responseBody).toString(), symbol);

    output.addHeader("content-type", "application/json");
    output ~= entry;
}

@endpoint
@route!("/api/v1/periodic-table/shells") void shellsHandler(Request req, Output output) {
    auto symbol = req.get.read("symbol");
    auto rs = client.get("http://web-data-source/shells.json");
    auto entry = findEntry((rs.responseBody).toString(), symbol);

    output.addHeader("content-type", "application/json");
    output ~= `{"shells": ` ~ entry ~ `}`;
}
