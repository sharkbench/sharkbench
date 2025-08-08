import std.socket;
import std.array;
import std.algorithm;
import std.typecons : Yes;
import std.format;
import std.conv;

class TCPServer {
    string host;
    ushort port;

    @disable this();

    this(string host, ushort port){
        this.host = host;
        this.port = port;
    }

    void start(){
        auto tcps = new TcpSocket(AddressFamily.INET);

        auto addr = new InternetAddress(host, port);

        tcps.bind(addr);

        tcps.listen(10);

        while(true){
            auto s = tcps.accept();

            char[1024] data;
            auto nbytes = s.receive(data);
            auto sp = data[0..nbytes].split("/?iterations=");
            if (sp.length < 2)
                continue;
            auto iter = sp[1].split(" ")[0];
            auto result = calcPi(iter.to!int);
            auto response = appender!string;
            response.reserve(30);
            response ~= "HTTP/1.1 200 OK";
            response ~= "\r\n\r\n";
            response ~= result;
            s.send(response[]);

            s.close();
        }
    }
}

void main()
{
    auto ts = new TCPServer("0.0.0.0", 3000);
    ts.start();
}

string calcPi(int iterations) {
    double pi = 0.0;
    double denominator = 1.0;
    double total_sum = 0.0;
    double alt_sum = 0.0;

    foreach(x; 0 .. iterations)
    {
        if (x % 2 == 0)
            pi += 1.0 / denominator;
        else
            pi -= 1.0 / denominator;
        denominator += 2.0;

        // custome
        total_sum += pi;
        switch (x % 3) {
            case 0:
                alt_sum += pi;
                break;
            case 1:
                alt_sum -= pi;
                break;
            default:
                alt_sum /= 2.0;
                break;
        }
    }
    return format!"%.16f;%.7f;%.16f"(pi * 4, total_sum, alt_sum);
}
