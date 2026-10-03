use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let iterations = line.parse::<usize>().unwrap();
        let result = calc_pi(iterations);
        writeln!(out, "{};{};{}", result.0, result.1, result.2).unwrap();
        out.flush().unwrap();
    }
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
