import arsd.cgi;
import arsd.http2 : Uri, HttpClient;
import arsd.jsvar;

struct Element {
    string name;
    ubyte number;
    ubyte group;
}

struct Shell {
    ubyte[] shells;
}

@DefaultFormat("json"):
class ApiV1 : WebObject {
    static HttpClient internClient;

    void makeClient() {
        if (internClient is null) {
            auto c = new HttpClient();
            c.keepAlive = true;
            internClient = c;
        }
    }

    this() {
        makeClient();
    }

    auto element(string symbol) {
        auto request = internClient.request(Uri("http://web-data-source/element.json"));
        request.send();
        auto response = request.waitForCompletion();
        auto entry = response.contentJson()[symbol];
        return entry.get!Element;
    }
    auto shells(string symbol) {
        auto request = internClient.request(Uri("http://web-data-source/shells.json"));
        request.send();
        auto response = request.waitForCompletion();
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
    auto server = RequestServer("0.0.0.0", 3000);
    server.serve!requestHandler();
}
