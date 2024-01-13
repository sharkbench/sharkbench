#include <kore/kore.h>
#include <kore/http.h>
#include <math.h>
#include <stdlib.h>

int	serve_pi(struct http_request *);

void calc_pi(int iterations, double *outPi, double *outSum, double *outCustomNumber) {
    double pi = 0.0;
    double denominator = 1.0;

    double sum = 0.0;
    double customNumber = 0.0;

    for (int x = 0; x < iterations; x++) {
        if (x % 2 == 0) {
            pi += 1 / denominator;
        } else {
            pi -= 1 / denominator;
        }
        denominator += 2;

        // custom calculations
        sum += pi;
        switch (x % 3) {
            case 0:
                customNumber += pi;
                break;
            case 1:
                customNumber -= pi;
                break;
            case 2:
                customNumber /= 2;
                break;
        }
    }

    pi *= 4;

    // Assign to output pointers
    *outPi = pi;
    *outSum = sum;
    *outCustomNumber = customNumber;
}

int serve_pi(struct http_request *req) {
    http_response_header(req, "content-type", "text/plain");

    // Populate GET parameters
//    http_populate_get(req);

    // Extracting query parameter "iterations"
//    int iterations = 0;
//    char *iterations_str;
//    int32_t value;
//    if (http_argument_get_int32(req, "iterations", &value)) {
//        iterations = value;
//    } else {
//        iterations = 0; // Default value or error handling
//    }

    // TODO: Parse correct "iterations" query parameter
    double pi, sum, customNumber;
    calc_pi(1000000000, &pi, &sum, &customNumber);

    char response[1024];
    snprintf(response, sizeof(response), "%.16f;%.7f;%.16f", pi, sum, customNumber);

    http_response(req, 200, response, strlen(response));

    return (KORE_RESULT_OK);
}
