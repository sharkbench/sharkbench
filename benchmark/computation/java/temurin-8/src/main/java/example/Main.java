package example;

import java.io.*;
import java.net.ServerSocket;
import java.net.Socket;

public class Main {
    public static void main(String[] args) {
        try (ServerSocket serverSocket = new ServerSocket(3000)) {
            System.out.println("Server started on port 3000");

            while (true) {
                try (Socket clientSocket = serverSocket.accept()) {
                    handleConnection(clientSocket);
                } catch (IOException e) {
                    System.out.println("Error handling client connection: " + e.getMessage());
                }
            }
        } catch (IOException e) {
            System.out.println("Could not listen on port 3000: " + e.getMessage());
        }
    }

    private static void handleConnection(Socket clientSocket) throws IOException {
        BufferedReader reader = new BufferedReader(new InputStreamReader(clientSocket.getInputStream()));
        PrintWriter writer = new PrintWriter(clientSocket.getOutputStream(), true);

        String requestLine = reader.readLine();
        if (requestLine == null || !requestLine.contains("/?iterations=")) {
            return;
        }

        String[] parts = requestLine.split("/\\?iterations=");
        if (parts.length != 2) {
            return;
        }

        String iterationsPart = parts[1].split(" ")[0];
        int iterations = Integer.parseInt(iterationsPart);
        double[] result = calcPi(iterations);

        String responseHeader = "HTTP/1.1 200 OK";
        String responseBody = result[0] + ";" + String.format("%.7f", result[1]) + ";" + result[2];

        writer.println(responseHeader);
        writer.println();
        writer.println(responseBody);
    }

    private static double[] calcPi(int iterations) {
        double pi = 0.0;
        double denominator = 1.0;
        double sum = 0.0;
        double customNumber = 0.0;

        for (int x = 0; x < iterations; x++) {
            if (x % 2 == 0) {
                pi += (1.0 / denominator);
            } else {
                pi -= (1.0 / denominator);
            }
            denominator += 2.0;

            sum += pi;
            switch (x % 3) {
                case 0:
                    customNumber += pi;
                    break;
                case 1:
                    customNumber -= pi;
                    break;
                default:
                    customNumber /= 2.0;
            }
        }
        pi = pi * 4;
        return new double[]{pi, sum, customNumber};
    }
}
