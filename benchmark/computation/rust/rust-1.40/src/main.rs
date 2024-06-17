use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();

    for stream in listener.incoming() {
        handle_connection(stream.unwrap());
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request = buf_reader.lines().next().unwrap().unwrap();

    let split = request.split("/?iterations=");
    if split.clone().count() != 2 {
        return;
    }
    let iterations = split.take(2).last().unwrap().split(" ").next().unwrap().parse::<usize>().unwrap();
    let result = calc_pi(iterations);
    let response_header = "HTTP/1.1 200 OK";
    let response_body = format!("{};{};{}", result.0, result.1, result.2);

    stream.write_all(format!("{}\r\n\r\n{}", response_header, response_body).as_bytes()).unwrap();
}

fn calc_pi(iterations: usize) -> (f64, f64, f64) {
    let mut pi = 0.0;
    let mut denominator = 1.0;
    let mut total_sum = 0.0;
    let mut alternating_sum = 0.0;
    for x in 0..iterations {
        if x % 2 == 0 {
            pi = pi + (1.0 / denominator);
        } else {
            pi = pi - (1.0 / denominator);
        }
        denominator = denominator + 2.0;

        // custom
        total_sum = total_sum + pi;
        match x % 3 {
            0 => alternating_sum = alternating_sum + pi,
            1 => alternating_sum = alternating_sum - pi,
            _ => alternating_sum /= 2.0,
        }
    }
    (pi * 4.0, total_sum, alternating_sum)
}
