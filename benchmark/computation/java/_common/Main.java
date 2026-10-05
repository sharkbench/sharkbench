package example;

import java.io.*;

public class Main {
    public static void main(String[] args) throws IOException {
        BufferedReader reader = new BufferedReader(new InputStreamReader(System.in));
        PrintStream out = System.out;

        String line;
        while ((line = reader.readLine()) != null) {
            line = line.trim();
            if (line.isEmpty()) {
                continue;
            }

            int iterations = Integer.parseInt(line);
            double[] result = calcPi(iterations);

            out.println(result[0] + ";" + String.format("%.7f", result[1]) + ";" + result[2]);
            out.flush();
        }
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
