const std = @import("std");
const File = std.Io.File;

pub fn main(init: std.process.Init) !void {
    const io = init.io;

    var stdin_buffer: [1024]u8 = undefined;
    var stdout_buffer: [1024]u8 = undefined;

    var stdin_reader = File.stdin().readerStreaming(io, &stdin_buffer);
    var stdout_writer = File.stdout().writerStreaming(io, &stdout_buffer);
    const stdin = &stdin_reader.interface;
    const stdout = &stdout_writer.interface;

    while (try stdin.takeDelimiter('\n')) |line| {
        const trimmed = std.mem.trim(u8, line, " \t\r");
        if (trimmed.len == 0) {
            continue;
        }
        const iterations = try std.fmt.parseUnsigned(u32, trimmed, 10);
        const result = calc_pi(iterations);
        try stdout.print("{d};{d};{d}\n", .{ result.pi, result.sum, result.alt_sum });
        try stdout.flush();
    }
}

fn calc_pi(iterations: u32) struct { pi: f64, sum: f64, alt_sum: f64 } {
    var pi: f64 = 0;
    var denominator: f64 = 1;
    var sum: f64 = 0;
    var alt_sum: f64 = 0;

    for (0..iterations) |i| {
        if (i % 2 == 0) {
            pi += (1 / denominator);
        } else {
            pi -= (1 / denominator);
        }
        denominator += 2;

        sum += pi;
        switch (i % 3) {
            0 => {
                alt_sum += pi;
            },
            1 => {
                alt_sum -= pi;
            },
            else => {
                alt_sum /= 2;
            },
        }
    }
    return .{ .pi = pi * 4, .sum = sum, .alt_sum = alt_sum };
}
