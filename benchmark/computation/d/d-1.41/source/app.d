import std.stdio;
import std.string : strip;
import std.format;
import std.conv;

void main()
{
    foreach (line; stdin.byLine())
    {
        auto trimmed = line.strip();
        if (trimmed.length == 0)
            continue;
        auto result = calcPi(trimmed.to!int);
        stdout.writeln(result);
        stdout.flush();
    }
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
