package example;

import org.eclipse.jetty.server.Server;
import org.eclipse.jetty.servlet.ServletContextHandler;
import org.eclipse.jetty.servlet.ServletHolder;
import org.mindrot.jbcrypt.BCrypt;

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

        context.addServlet(new ServletHolder(new BcryptServlet()), "/*");

        server.start();
        System.out.println("Running!");
        server.join();
    }

    public static class BcryptServlet extends HttpServlet {
        @Override
        protected void doGet(HttpServletRequest req, HttpServletResponse resp) throws IOException {
            String password = req.getParameter("password");
            String salt = req.getParameter("salt");
            System.out.println("salt: " + BCrypt.gensalt(12));
            String hashed = BCrypt.hashpw(password, salt);
            resp.getWriter().println(hashed);
        }
    }
}
