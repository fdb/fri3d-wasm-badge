const std = @import("std");

pub fn build(b: *std.Build) void {
    // ========================================================================
    // Pure Zig WASM Apps (no native dependencies needed)
    // ========================================================================
    const zig_wasm_target = b.resolveTargetQuery(.{
        .cpu_arch = .wasm32,
        .os_tag = .freestanding,
    });

    // Create shared modules for Zig WASM apps
    const platform_module = b.createModule(.{
        .root_source_file = b.path("src/sdk/zig/platform.zig"),
        .target = zig_wasm_target,
        .optimize = .ReleaseSmall,
    });

    const imgui_module = b.createModule(.{
        .root_source_file = b.path("src/sdk/zig/imgui.zig"),
        .target = zig_wasm_target,
        .optimize = .ReleaseSmall,
        .imports = &.{
            .{ .name = "platform", .module = platform_module },
        },
    });

    // Circles app in pure Zig
    const circles_zig = b.addExecutable(.{
        .name = "circles_zig",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/apps/circles_zig/main.zig"),
            .target = zig_wasm_target,
            .optimize = .ReleaseSmall,
            .imports = &.{
                .{ .name = "platform", .module = platform_module },
            },
        }),
    });

    circles_zig.rdynamic = true;
    circles_zig.entry = .disabled;

    b.installArtifact(circles_zig);

    // Test UI app in pure Zig (uses IMGUI)
    const test_ui_zig = b.addExecutable(.{
        .name = "test_ui_zig",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/apps/test_ui_zig/main.zig"),
            .target = zig_wasm_target,
            .optimize = .ReleaseSmall,
            .imports = &.{
                .{ .name = "platform", .module = platform_module },
                .{ .name = "imgui", .module = imgui_module },
            },
        }),
    });

    test_ui_zig.rdynamic = true;
    test_ui_zig.entry = .disabled;

    b.installArtifact(test_ui_zig);

    // ========================================================================
    // WASM build step
    // ========================================================================
    const wasm_step = b.step("wasm", "Build only WASM apps");
    wasm_step.dependOn(&b.addInstallArtifact(circles_zig, .{}).step);
    wasm_step.dependOn(&b.addInstallArtifact(test_ui_zig, .{}).step);

    // ========================================================================
    // Web build step - build WASM apps and copy to www/
    // ========================================================================
    const web_step = b.step("web", "Build web version (WASM apps + web files)");

    // Web step depends on building the WASM apps
    web_step.dependOn(wasm_step);

    // Copy web platform files
    const copy_html = b.addInstallFile(b.path("src/ports/web/index.html"), "../www/index.html");
    const copy_js = b.addInstallFile(b.path("src/ports/web/platform.js"), "../www/platform.js");

    web_step.dependOn(&copy_html.step);
    web_step.dependOn(&copy_js.step);

    // Copy all WASM apps to www/
    const wasm_apps_web = [_][]const u8{ "circles_zig", "test_ui_zig" };
    for (wasm_apps_web) |app_name| {
        const dest_path = b.fmt("../www/{s}.wasm", .{app_name});
        const copy_wasm = b.addInstallFile(b.path(b.fmt("zig-out/bin/{s}.wasm", .{app_name})), dest_path);
        web_step.dependOn(&copy_wasm.step);
    }
}
