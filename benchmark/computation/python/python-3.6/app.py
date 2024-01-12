import http.server
import socketserver
from urllib.parse import urlparse, parse_qs

PORT = 3000

class SimpleHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        # Parse query parameters
        query_components = parse_qs(urlparse(self.path).query)
        iterations = int(query_components.get('iterations', [1])[0])

        # Calculate
        [pi, sum, custom_number] = calc_pi(iterations)

        # Send the HTTP response
        self.send_response(200)
        self.send_header("Content-type", "text/plain")
        self.end_headers()
        self.wfile.write(f"{pi};{sum};{custom_number}".encode())

def calc_pi(iterations):
    pi = 0.0
    denominator = 1.0
    sum = 0.0
    custom_number = 0.0
    for x in range(iterations):
        if x % 2 == 0:
            pi = pi + (1 / denominator)
        else:
            pi = pi - (1 / denominator)
        denominator = denominator + 2

        # custom
        sum += pi
        mod_3 = x % 3
        if mod_3 == 0:
            custom_number += pi
        elif mod_3 == 1:
            custom_number -= pi
        else:
            custom_number /= 2
    pi = pi * 4
    return [pi, sum, custom_number]

if __name__ == "__main__":
    with socketserver.TCPServer(("", PORT), SimpleHTTPRequestHandler) as httpd:
        print(f"Serving at port {PORT}")
        httpd.serve_forever()
