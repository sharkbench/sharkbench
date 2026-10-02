import vibe.core.core : runApplication;
import vibe.http.server;
import vibe.http.client;
import vibe.http.router;
import vibe.data.json;

import std.stdio : writeln;

void elementHandler(scope HTTPServerRequest req, scope HTTPServerResponse res)
{
	auto symbol = req.query().get("symbol");
	requestHTTP("http://127.0.0.1:5002/element.json",
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
    requestHTTP("http://127.0.0.1:5002/shells.json",
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
        settings.port = 5001;
        settings.bindAddresses = ["0.0.0.0"];
        listenHTTP(settings, router);
    }
    catch (Exception e) {
        assert(false, e.msg);
    }
	runApplication();
}
