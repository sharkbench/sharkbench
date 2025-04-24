const std = @import("std");
const net = std.net;
const http = std.http;

pub fn main() !void {
    var gpa = std.heap.DebugAllocator(.{}).init;
    const allocator = gpa.allocator();
    defer _ = gpa.deinit();

    const addr = net.Address.parseIp4("0.0.0.0", 3000) catch unreachable;
    var server = try addr.listen(.{});
    start_server(allocator, &server);
}

fn start_server(allocator: std.mem.Allocator, server: *net.Server) void {
    while (true) {
        var connection = server.accept() catch unreachable;
        defer connection.stream.close();

        var read_buffer: [1024]u8 = undefined;

        var http_server = http.Server.init(connection, &read_buffer);

        var request = http_server.receiveHead() catch unreachable;
        handle_request(allocator, &request) catch unreachable;
    }
}

fn handle_request(allocator: std.mem.Allocator, request: *http.Server.Request) !void {
    const parameter = request.head.target[13..];
    const iterations = try std.fmt.parseUnsigned(u32, parameter, 10);
    const result = calc_pi(iterations);
    const formatted_result = try std.fmt.allocPrint(allocator, "{d};{d};{d}", .{ result.pi, result.sum, result.alt_sum });
    try request.respond(formatted_result, .{});
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
