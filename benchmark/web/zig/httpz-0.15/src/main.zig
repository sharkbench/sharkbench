const std = @import("std");
const httpz = @import("httpz");

const Handler = struct {
    client: std.http.Client,
};

const ElementResponse = struct {
    name: []const u8,
    number: i64,
    group: i64,
};

const ShellsResponse = struct {
    shells: []i64,
};

fn getElement(handler: *Handler, req: *httpz.Request, res: *httpz.Response) !void {
    const query = try req.query();
    const symbol = query.get("symbol").?;

    const uri = try std.Uri.parse("http://web-data-source/element.json");

    var data_req = try handler.client.request(.GET, uri, .{});
    defer data_req.deinit();

    try data_req.sendBodiless();
    var data_res = try data_req.receiveHead(&.{});

    var reader_buffer: [9 * 1024]u8 = undefined;
    const body_reader = data_res.reader(&reader_buffer);
    const response_body = try body_reader.allocRemaining(res.arena, .limited(reader_buffer.len));

    // use std.json.parseFromSliceLeaky because an arena allocator is used
    const parsed = try std.json.parseFromSliceLeaky(
        std.json.Value,
        res.arena,
        response_body,
        .{},
    );

    // const element_object = parsed.object.get(symbol).?;
    const element_object = parsed.object.get(symbol).?;
    const element_res = ElementResponse{
        .name = element_object.object.get("name").?.string,
        .number = element_object.object.get("number").?.integer,
        .group = element_object.object.get("group").?.integer,
    };

    res.status = 200;
    try res.json(element_res, .{});
}

fn getShells(handler: *Handler, req: *httpz.Request, res: *httpz.Response) !void {
    const query = try req.query();
    const symbol = query.get("symbol").?;

    const uri = try std.Uri.parse("http://web-data-source/shells.json");

    var data_req = try handler.client.request(.GET, uri, .{});
    defer data_req.deinit();

    try data_req.sendBodiless();
    var data_res = try data_req.receiveHead(&.{});

    var reader_buffer: [4 * 1024]u8 = undefined;
    const body_reader = data_res.reader(&reader_buffer);
    const body = try body_reader.allocRemaining(res.arena, .limited(reader_buffer.len));

    // use std.json.parseFromSliceLeaky because an arena allocator is used
    const parsed = try std.json.parseFromSliceLeaky(
        std.json.Value,
        res.arena,
        body,
        .{},
    );

    // const shells_object = parsed.object.get(symbol).?.array;
    const shells_object = parsed.object.get(symbol).?.array;
    var shells = try res.arena.alloc(i64, shells_object.items.len);

    for (shells_object.items, 0..) |item, index| {
        shells[index] = item.integer;
    }

    const shells_res = ShellsResponse{
        .shells = shells,
    };

    res.status = 200;
    try res.json(shells_res, .{});
}

pub fn main() !void {
    var gpa = std.heap.DebugAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var client = std.http.Client{ .allocator = allocator };
    defer client.deinit();

    var handler = Handler{ .client = client };
    var server = try httpz.Server(*Handler).init(allocator, .{
        .address = .{
            .addr = .initIp4(.{ 0, 0, 0, 0 }, 3000),
        },
    }, &handler);

    defer {
        // clean shutdown, finishes serving any live request
        server.stop();
        server.deinit();
    }

    var router = try server.router(.{});
    router.get("/api/v1/periodic-table/element", getElement, .{});
    router.get("/api/v1/periodic-table/shells", getShells, .{});

    // blocks
    try server.listen();
}
