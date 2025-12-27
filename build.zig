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

    // Copy web platform files (these are source files, use b.path)
    const copy_html = b.addInstallFile(b.path("src/ports/web/index.html"), "../www/index.html");
    const copy_js = b.addInstallFile(b.path("src/ports/web/platform.js"), "../www/platform.js");

    web_step.dependOn(&copy_html.step);
    web_step.dependOn(&copy_js.step);

    // Copy WASM artifacts to www/ (use getEmittedBin for build outputs)
    const copy_circles = b.addInstallFile(circles_zig.getEmittedBin(), "../www/circles_zig.wasm");
    const copy_test_ui = b.addInstallFile(test_ui_zig.getEmittedBin(), "../www/test_ui_zig.wasm");

    web_step.dependOn(&copy_circles.step);
    web_step.dependOn(&copy_test_ui.step);
}
