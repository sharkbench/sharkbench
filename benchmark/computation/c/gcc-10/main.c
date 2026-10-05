#include <stdio.h>
#include <stdlib.h>

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

int main() {
    char line[64];

    while (fgets(line, sizeof(line), stdin) != NULL) {
        int iterations = atoi(line);
        if (iterations <= 0) {
            continue;
        }

        double pi, sum, customNumber;
        calc_pi(iterations, &pi, &sum, &customNumber);

        printf("%.16f;%.7f;%.16f\n", pi, sum, customNumber);
        fflush(stdout);
    }

    return 0;
}
