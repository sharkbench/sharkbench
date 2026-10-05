#if canImport(Glibc)
import Glibc
#else
import Darwin
#endif

func calculatePi(iterations: Int) -> (Double, Double, Double) {
    var pi = 0.0
    var denominator = 1.0
    var sum = 0.0
    var customNumber = 0.0

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
        case 0: customNumber += pi
        case 1: customNumber -= pi
        default: customNumber /= 2
        }
    }

    return (pi * 4, sum, customNumber)
}

while let line = readLine() {
    let input = line.filter { !$0.isWhitespace }
    if input.isEmpty {
        continue
    }
    let (pi, sum, customNumber) = calculatePi(iterations: Int(input)!)
    print("\(pi);\(sum);\(customNumber)")
    fflush(stdout)
}
