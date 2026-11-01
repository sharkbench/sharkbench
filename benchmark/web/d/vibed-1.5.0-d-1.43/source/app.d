import vibe.core.core : runApplication;
import vibe.http.server;
import vibe.http.client : requestHTTP;
import vibe.http.router;
import vibe.data.json : deserializeJson;
import vibe.stream.operations : readAllUTF8;

struct DataSourceElement {
    string name;
    ubyte number;
    ubyte group;
}

struct ElementResponse {
    string name;
    ubyte number;
    ubyte group;
}

struct ShellsResponse {
    ubyte[] shells;
}

void elementHandler(
    scope HTTPServerRequest req,
    scope HTTPServerResponse res)
{
    auto symbol = req.query.get("symbol");

    requestHTTP(
        "http://127.0.0.1:5001/element.json",
        (scope creq) {
            creq.method = HTTPMethod.GET;
        },
        (scope cres) {
            // Read the body and deserialize directly into typed data.
            auto body = cres.bodyReader.readAllUTF8();
            auto data =
                deserializeJson!(DataSourceElement[string])(body);

            auto entry = data[symbol];

            res.writeJsonBody(ElementResponse(
                entry.name,
                entry.number,
                entry.group
            ));
        }
    );
}

void shellsHandler(
    scope HTTPServerRequest req,
    scope HTTPServerResponse res)
{
    auto symbol = req.query.get("symbol");

    requestHTTP(
        "http://127.0.0.1:5001/shells.json",
        (scope creq) {
            creq.method = HTTPMethod.GET;
        },
        (scope cres) {
            auto body = cres.bodyReader.readAllUTF8();
            auto data = deserializeJson!(ubyte[][string])(body);

            auto entry = data[symbol];

            res.writeJsonBody(ShellsResponse(entry));
        }
    );
}

void main()
{
    auto router = new URLRouter;

    router.get("/api/v1/periodic-table/element", &elementHandler);
    router.get("/api/v1/periodic-table/shells", &shellsHandler);

    auto settings = new HTTPServerSettings;
    settings.options |= HTTPServerOption.reusePort;
    settings.port = 5001;
    settings.bindAddresses = ["0.0.0.0"];

    listenHTTP(settings, router);
    runApplication();
}
