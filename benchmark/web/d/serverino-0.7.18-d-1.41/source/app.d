import std.datetime;
import std.json;
import std.conv : to;

import serverino;
import requests : req = Request;

mixin ServerinoMain;

__gshared req client = req();

@onServerInit ServerinoConfig configure()
{
	return ServerinoConfig
		.create()
        .setHttpTimeout(15.seconds)
        .enableKeepAlive(180.seconds)
   		.addListener("0.0.0.0", 3000)
		.setWorkers(10);
}

@endpoint
@route!("/api/v1/periodic-table/element") void elementHandler(Request req, Output output) {
    auto symbol = req.get.read("symbol");
    auto rs = client.get("http://web-data-source/element.json");
    auto entry = parseJSON((rs.responseBody).toString())[symbol];

    output.addHeader("content-type", "application/json");
    output ~= entry.toString();
}

@endpoint
@route!("/api/v1/periodic-table/shells") void shellsHandler(Request req, Output output) {
    auto symbol = req.get.read("symbol");
    auto rs = client.get("http://web-data-source/shells.json");
    auto entry = parseJSON((rs.responseBody).toString())[symbol];

    output.addHeader("content-type", "application/json");
    output ~= `{"shells": ` ~ entry.to!string ~ `}`;
}
