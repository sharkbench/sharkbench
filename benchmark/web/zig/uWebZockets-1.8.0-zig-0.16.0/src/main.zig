const std = @import("std");
const uz = @import("uWebZockets");

const worker_count = 4;

const Context = struct {
    client: std.http.Client,
    body_buffer: [16 * 1024]u8 = undefined,
    arena_buffer: [256 * 1024]u8 = undefined,
};

var contexts: [worker_count]Context = undefined;

fn symbol(req: *uz.Request) ?[]const u8 {
    const params = req.query_params() catch return null;
    return params.get("symbol");
}

fn fetchJson(ctx: *Context, path: []const u8) ?[]const u8 {
    const uri = std.Uri.parse(path) catch return null;

    var request = ctx.client.request(.GET, uri, .{}) catch return null;
    defer request.deinit();

    request.sendBodiless() catch return null;
    var response = request.receiveHead(&.{}) catch return null;

    var transfer_buffer: [16 * 1024]u8 = undefined;
    const body_reader = response.reader(&transfer_buffer);

    const len = body_reader.readSliceShort(&ctx.body_buffer) catch return null;
    return ctx.body_buffer[0..len];
}

fn elementHandler(context: *anyopaque, req: *uz.Request, res: *uz.Response) void {
    const ctx: *Context = @ptrCast(@alignCast(context));
    const element_symbol = symbol(req) orelse return res.end("400 Bad Request", "") catch {};

    const body = fetchJson(ctx, "http://127.0.0.1:5002/element.json") orelse return res.end("500 Internal Server Error", "") catch {};

    var arena = std.heap.FixedBufferAllocator.init(&ctx.arena_buffer);
    const parsed = std.json.parseFromSliceLeaky(
        std.json.Value,
        arena.allocator(),
        body,
        .{},
    ) catch return res.end("500 Internal Server Error", "") catch {};

    const element = parsed.object.get(element_symbol) orelse return res.end("404 Not Found", "") catch {};
    const object = element.object;

    var buffer: [256]u8 = undefined;
    res.json_buf(.{
        .name = object.get("name").?.string,
        .number = object.get("number").?.integer,
        .group = object.get("group").?.integer,
    }, &buffer) catch {};
}

fn shellsHandler(context: *anyopaque, req: *uz.Request, res: *uz.Response) void {
    const ctx: *Context = @ptrCast(@alignCast(context));
    const element_symbol = symbol(req) orelse return res.end("400 Bad Request", "") catch {};

    const body = fetchJson(ctx, "http://127.0.0.1:5002/shells.json") orelse return res.end("500 Internal Server Error", "") catch {};

    var arena = std.heap.FixedBufferAllocator.init(&ctx.arena_buffer);
    const parsed = std.json.parseFromSliceLeaky(
        std.json.Value,
        arena.allocator(),
        body,
        .{},
    ) catch return res.end("500 Internal Server Error", "") catch {};

    const shells = parsed.object.get(element_symbol) orelse return res.end("404 Not Found", "") catch {};

    var items: [16]i64 = undefined;
    for (shells.array.items, 0..) |item, index| items[index] = item.integer;

    var buffer: [256]u8 = undefined;
    res.json_buf(.{ .shells = items[0..shells.array.items.len] }, &buffer) catch {};
}

pub fn main(init: std.process.Init) !void {
    for (&contexts) |*ctx| {
        ctx.* = Context{ .client = .{ .io = init.io, .allocator = init.gpa } };
    }

    var group = try uz.Server.builder(init.io)
        .with_max_clients(64)
        .with_write_queue_size(16 * 1024)
        .with_dev_log(false)
        .build_cluster(std.heap.page_allocator, worker_count, .{});
    defer group.deinit();

    const Cluster = @TypeOf(group);
    try group.configure(struct {
        fn routes(worker: *Cluster.Worker, index: usize) !void {
            _ = try worker.get_context("/api/v1/periodic-table/element", &contexts[index], elementHandler);
            _ = try worker.get_context("/api/v1/periodic-table/shells", &contexts[index], shellsHandler);
        }
    }.routes);

    try group.listen("0.0.0.0", 5001);
    try group.run();
}
