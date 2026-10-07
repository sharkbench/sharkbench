const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const exe_mod = b.createModule(.{
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    const exe = b.addExecutable(.{
        .name = "benchmark",
        .root_module = exe_mod,
    });

    b.installArtifact(exe);

    const uz = b.dependency("uWebZockets", .{
        .target = target,
        .optimize = optimize,
    });
    exe.root_module.addImport("uWebZockets", uz.module("uWebZockets"));
}
