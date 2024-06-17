#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <arpa/inet.h>

#define PORT 3000

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

void handle_connection(int client_socket) {
    char buffer[1024] = {0};
    read(client_socket, buffer, 1024);

    int iterations = atoi(strstr(buffer, "iterations=") + 11);

    double pi, sum, customNumber;
    calc_pi(iterations, &pi, &sum, &customNumber);

    char response[1024];
    sprintf(response, "HTTP/1.1 200 OK\r\n\r\n%.16f;%.7f;%.16f\n", pi, sum, customNumber);
    write(client_socket, response, strlen(response));

    close(client_socket);
}

int main() {
    int server_fd, new_socket;
    struct sockaddr_in address;
    int addrlen = sizeof(address);

    if ((server_fd = socket(AF_INET, SOCK_STREAM, 0)) == 0) {
        perror("socket failed");
        exit(EXIT_FAILURE);
    }

    address.sin_family = AF_INET;
    address.sin_addr.s_addr = INADDR_ANY;
    address.sin_port = htons(PORT);

    if (bind(server_fd, (struct sockaddr *)&address, sizeof(address)) < 0) {
        perror("bind failed");
        exit(EXIT_FAILURE);
    }

    if (listen(server_fd, 10) < 0) {
        perror("listen failed");
        exit(EXIT_FAILURE);
    }

    printf("Listening on port %d...\n", PORT);
    while (1) {
        if ((new_socket = accept(server_fd, (struct sockaddr *)&address, (socklen_t*)&addrlen)) < 0) {
            perror("accept failed");
            exit(EXIT_FAILURE);
        }
        handle_connection(new_socket);
    }

    return 0;
}
