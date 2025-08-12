import arsd.cgi;
import arsd.http2 : ICache, Uri, HttpClient, HttpResponse, HttpRequestParameters;
import arsd.jsvar;

struct Element {
    string name;
    ubyte number;
    ubyte group;
}

struct Shell {
    ubyte[] shells;
}

class myCache: ICache {
    __gshared HttpResponse[HttpRequestParameters] aa;

    const(HttpResponse)* getCachedResponse(HttpRequestParameters request) {
        return &aa[request];
    }

    bool cacheResponse(HttpRequestParameters request, HttpResponse response) {
        aa[request] = response;
        return true;
    }
}


class ApiV1 : WebObject {
    static HttpClient internClient;
    myCache internCache;

    void makeClient() {
        if (internClient is null) {
            auto cc = new myCache();
            internCache = cc;
            auto c = new HttpClient(internCache);
            c.keepAlive = true;
            internClient = c;
        }
    }

    this() {
        makeClient();
    }

    @DefaultFormat("json"):
    auto element(string symbol) {
        auto request = internClient.request(Uri("http://web-data-source/element.json"));
        request.send();
        auto response = request.waitForCompletion();
        auto entry = response.contentJson()[symbol];
        return entry.get!Element;
    }

    @DefaultFormat("json"):
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
    //server.numberOfThreads = 100;
    server.serve!requestHandler();
}
