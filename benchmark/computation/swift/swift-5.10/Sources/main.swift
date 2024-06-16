import Vapor

var env = try Environment.detect()
try LoggingSystem.bootstrap(from: &env)
let app = Application(env)
defer { app.shutdown() }
app.middleware = .init()

app.logger.logLevel = .critical

app.get { req -> String in
    let iterations = req.query[Int.self, at: "iterations"] ?? 0
    let result = calculatePi(iterations: iterations)
    return "\(result.pi);\(result.sum);\(result.customNumber)"
}

func calculatePi(iterations: Int) -> (pi: Double, sum: Double, customNumber: Double) {
    var pi: Double = 0.0
    var denominator: Double = 1.0
    var sum: Double = 0.0
    var customNumber: Double = 0.0

    for i in 0..<iterations {
        if i % 2 == 0 {
            pi += 1 / denominator
        } else {
            pi -= 1 / denominator
        }
        denominator += 2

        // Custom calculations
        sum += pi
        switch i % 3 {
        case 0:
            customNumber += pi
        case 1:
            customNumber -= pi
        case 2:
            customNumber /= 2
        default:
            break
        }
    }

    pi *= 4
    return (pi: pi, sum: sum, customNumber: customNumber)
}

app.http.server.configuration.hostname = "0.0.0.0"
app.http.server.configuration.port = 3000

try app.run()
