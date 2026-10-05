package main

import (
    "bufio"
    "fmt"
    "os"
    "strconv"
    "strings"
)

func main() {
    // os.Stdout is unbuffered, so every response is written immediately
    scanner := bufio.NewScanner(os.Stdin)
    for scanner.Scan() {
        line := strings.TrimSpace(scanner.Text())
        if line == "" {
            continue
        }
        iterations, err := strconv.Atoi(line)
        if err != nil {
            fmt.Fprintf(os.Stderr, "Invalid iterations: %s\n", line)
            os.Exit(1)
        }
        result := pi(iterations)
        fmt.Printf("%.16f;%.7f;%.16f\n", result[0], result[1], result[2])
    }
}

func pi(iterations int) []float64 {
    var pi float64 = 0
    var denominator float64 = 1

    var sum float64 = 0
    var customNumber float64 = 0

    for i := 0; i < iterations; i++ {
        if i%2 == 0 {
            pi += (1 / denominator)
        } else {
            pi -= (1 / denominator)
        }
        denominator += 2

        // custom
        sum += pi
        switch i % 3 {
        case 0:
            customNumber += pi
        case 1:
            customNumber -= pi
        case 2:
            customNumber /= 2
        }
    }

    pi *= 4
    return []float64{pi, sum, customNumber}
}
