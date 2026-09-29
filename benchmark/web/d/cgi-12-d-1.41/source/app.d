import arsd.cgi;
import arsd.http2 : get, Uri, HttpClient, HttpResponse, HttpRequestParameters;
import arsd.jsvar;

struct Element {
    string name;
    ubyte number;
    ubyte group;
}

struct Shell {
    ubyte[] shells;
}

class ApiV1 : WebObject {
    @DefaultFormat("json"):
    auto element(string symbol) {
        auto response = get(Uri("http://127.0.0.1:5002/element.json")).waitForCompletion;
        auto entry = response.contentJson()[symbol];
        return entry.get!Element;
    }

    @DefaultFormat("json"):
    auto shells(string symbol) {
        auto response = get(Uri("http://127.0.0.1:5002/shells.json")).waitForCompletion();
        auto entry = response.contentJson()[symbol];
        return Shell(entry.get!(ubyte[]));
    }
}

void requestHandler(Cgi cgi) {
    cgi.dispatcher!(
        "/api/v1/periodic-table/".serveApi!ApiV1
    );
}

void main() {
    auto server = RequestServer("0.0.0.0", 5001);
    server.serve!requestHandler();
}
