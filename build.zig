const std = @import("std");

pub fn build(b: *std.Build) void {
    const optimize = b.standardOptimizeOption(.{});
    const target = b.standardTargetOptions(.{
        .default_target = .{ .cpu_arch = .wasm32, .os_tag = .wasi },
    });

    const apps_step = b.step("apps", "Build all Zig WASM apps");

    addWasmApp(b, apps_step, .{
        .name = "circles",
        .root_source_file = "zig/apps/circles/main.zig",
    }, target, optimize);
}

const WasmApp = struct {
    name: []const u8,
    root_source_file: []const u8,
};

fn addWasmApp(
    b: *std.Build,
    parent_step: *std.Build.Step,
    app: WasmApp,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
) void {
    const exe = b.addExecutable(.{
        .name = app.name,
        .root_source_file = .{ .path = app.root_source_file },
        .target = target,
        .optimize = optimize,
    });

    exe.single_threaded = true;

    const install_artifact = b.addInstallArtifact(exe, .{});
    parent_step.dependOn(&install_artifact.step);
}
