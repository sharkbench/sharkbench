using System;
using System.Net;
using System.Threading.Tasks;

class Program
{
    const int port = 3000;

    static async Task Main()
    {
        HttpListener listener = new HttpListener();
        listener.Prefixes.Add($"http://*:{port}/");
        listener.Start();
        Console.WriteLine($"Running on port {port}");

        while (true)
        {
            HttpListenerContext context = await listener.GetContextAsync();
            HttpListenerRequest request = context.Request;
            HttpListenerResponse response = context.Response;

            string iterationsParam = request.QueryString["iterations"];
            int iterations = int.Parse(iterationsParam);
            var result = CalculatePi(iterations);

            string responseString = $"{result[0]};{result[1]};{result[2]}";
            byte[] buffer = System.Text.Encoding.UTF8.GetBytes(responseString);

            response.ContentLength64 = buffer.Length;
            System.IO.Stream output = response.OutputStream;
            await output.WriteAsync(buffer, 0, buffer.Length);
            output.Close();
        }
    }

    static double[] CalculatePi(int iterations)
    {
        double pi = 0.0;
        double denominator = 1.0;
        double sum = 0.0;
        double customNumber = 0.0;

        for (int x = 0; x < iterations; x++)
        {
            if (x % 2 == 0)
            {
                pi += (1 / denominator);
            }
            else
            {
                pi -= (1 / denominator);
            }
            denominator += 2;

            // custom
            sum += pi;
            switch (x % 3)
            {
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
        return new double[] { pi, sum, customNumber };
    }
}
