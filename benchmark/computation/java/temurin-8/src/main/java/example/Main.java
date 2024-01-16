package example;

import org.eclipse.jetty.server.Server;
import org.eclipse.jetty.servlet.ServletContextHandler;
import org.eclipse.jetty.servlet.ServletHolder;

import javax.servlet.http.HttpServlet;
import javax.servlet.http.HttpServletRequest;
import javax.servlet.http.HttpServletResponse;
import java.io.IOException;

public class Main {

    public static void main(String[] args) throws Exception {
        Server server = new Server(3000);

        ServletContextHandler context = new ServletContextHandler(ServletContextHandler.SESSIONS);
        context.setContextPath("/");
        server.setHandler(context);

        context.addServlet(new ServletHolder(new MyServlet()), "/*");

        server.start();
        System.out.println("Running!");
        server.join();
    }

    public static class MyServlet extends HttpServlet {
        @Override
        protected void doGet(HttpServletRequest req, HttpServletResponse resp) throws IOException {
            String iterations = req.getParameter("iterations");
            double[] result = pi(Integer.parseInt(iterations));
            resp.getWriter().print(result[0] + ";" + String.format("%.7f", result[1]) + ";" + result[2]);
        }

        private double[] pi(int iterations) {
            double pi = 0.0;
            double denominator = 1.0;
            double sum = 0.0;
            double customNumber = 0.0;
            for (int x = 0; x < iterations; x++) {
                if (x % 2 == 0) {
                    pi = pi + (1 / denominator);
                } else {
                    pi = pi - (1 / denominator);
                }
                denominator = denominator + 2;

                // custom
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
            pi = pi * 4;
            return new double[]{pi, sum, customNumber};
        }
    }
}
