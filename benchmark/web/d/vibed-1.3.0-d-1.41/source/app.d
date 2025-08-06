import vibe.core.core : runApplication;
import vibe.http.server;
import vibe.http.client;
import vibe.http.router;
import vibe.data.json;

import std.stdio : writeln;

void elementHandler(scope HTTPServerRequest req, scope HTTPServerResponse res)
{
	auto symbol = req.query().get("symbol");
	requestHTTP("http://web-data-source/element.json",
	    (scope creq) {
		    creq.method = HTTPMethod.GET;
	    },
	    (scope cres) {
			auto entry = cres.readJson()[symbol];
			res.writeJsonBody(entry);
	    }
	);
}

void shellsHandler(scope HTTPServerRequest req, scope HTTPServerResponse res)
{
    auto symbol = req.query().get("symbol");
    requestHTTP("http://web-data-source/shells.json",
		(scope creq) {
            creq.method = HTTPMethod.GET;
        },
        (scope cres) {
            auto entry = cres.readJson()[symbol];
            res.writeJsonBody(Json(["shells": entry]));
        }
    );
}

void main()
{
    try {
        auto router = new URLRouter;
        router.get("/api/v1/periodic-table/element", &elementHandler);
        router.get("/api/v1/periodic-table/shells", &shellsHandler);
        auto settings = new HTTPServerSettings;
        settings.options |= HTTPServerOption.reusePort;
        settings.port = 3000;
        settings.bindAddresses = ["0.0.0.0"];
        listenHTTP(settings, router);
    }
    catch (Exception e) {
        assert(false, e.msg);
    }
	runApplication();
}
