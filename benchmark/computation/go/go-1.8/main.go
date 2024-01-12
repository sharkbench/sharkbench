package main

import (
    "fmt"
    "log"
    "net/http"
    "strconv"
)

const port = 3000

func main() {
    http.HandleFunc("/", handleRequest)

    addr := fmt.Sprintf(":%d", port)
    fmt.Printf("Running on port %d\n", port)
    log.Fatal(http.ListenAndServe(addr, nil))
}

func handleRequest(w http.ResponseWriter, r *http.Request) {
    query := r.URL.Query()
    iterationsStr := query.Get("iterations")
    iterations, _ := strconv.Atoi(iterationsStr)
    result := pi(iterations)
    fmt.Fprintf(w, "%.16f;%.7f;%.16f", result[0], result[1], result[2])
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
