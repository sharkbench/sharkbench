const std = @import("std");
const net = std.Io.net;
const http = std.http;

pub fn main(init: std.process.Init) !void {
    const io = init.io;
    const addr = try net.IpAddress.parseIp4("0.0.0.0", 3000);
    var server = try addr.listen(io, .{ .reuse_address = true });
    defer server.deinit(io);

    try start_server(io, &server);
}

fn start_server(io: std.Io, server: *net.Server) !void {
    while (true) {
        var connection = try server.accept(io);
        defer connection.close(io);

        var recv_buffer: [1024]u8 = undefined;
        var send_buffer: [1024]u8 = undefined;

        var stream_reader = connection.reader(io, &recv_buffer);
        var stream_writer = connection.writer(io, &send_buffer);

        var http_server = http.Server.init(&stream_reader.interface, &stream_writer.interface);

        var request = try http_server.receiveHead();
        try handle_request(&request);
    }
}

fn handle_request(request: *http.Server.Request) !void {
    const parameter = request.head.target[13..];
    if (!std.mem.eql(u8, request.head.target[0..13], "/?iterations=")) {
        std.log.err("Expected request parameter as \"/?iterations=<number>\", got {s}", .{request.head.target});
    }
    const iterations = try std.fmt.parseUnsigned(u32, parameter, 10);
    const result = calc_pi(iterations);
    var buf: [1024]u8 = undefined;
    const formatted_result = try std.fmt.bufPrint(&buf, "{d};{d};{d}", .{ result.pi, result.sum, result.alt_sum });
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
