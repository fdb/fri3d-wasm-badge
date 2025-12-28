const std = @import("std");

pub fn build(b: *std.Build) void {
    // ========================================================================
    // Native target options (for desktop)
    // ========================================================================
    const native_target = b.standardTargetOptions(.{});
    const native_optimize = b.standardOptimizeOption(.{});

    // ========================================================================
    // Pure Zig WASM Apps (no native dependencies needed)
    // ========================================================================
    const zig_wasm_target = b.resolveTargetQuery(.{
        .cpu_arch = .wasm32,
        .os_tag = .freestanding,
    });

    // Create shared modules for Zig WASM apps
    const platform_module_wasm = b.createModule(.{
        .root_source_file = b.path("src/sdk/zig/platform.zig"),
        .target = zig_wasm_target,
        .optimize = .ReleaseSmall,
    });

    const imgui_module_wasm = b.createModule(.{
        .root_source_file = b.path("src/sdk/zig/imgui.zig"),
        .target = zig_wasm_target,
        .optimize = .ReleaseSmall,
        .imports = &.{
            .{ .name = "platform", .module = platform_module_wasm },
        },
    });

    // Circles app in pure Zig (WASM)
    const circles_zig = b.addExecutable(.{
        .name = "circles_zig",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/apps/circles_zig/main.zig"),
            .target = zig_wasm_target,
            .optimize = .ReleaseSmall,
            .imports = &.{
                .{ .name = "platform", .module = platform_module_wasm },
            },
        }),
    });

    circles_zig.rdynamic = true;
    circles_zig.entry = .disabled;

    b.installArtifact(circles_zig);

    // Test UI app in pure Zig (WASM, uses IMGUI)
    const test_ui_zig = b.addExecutable(.{
        .name = "test_ui_zig",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/apps/test_ui_zig/main.zig"),
            .target = zig_wasm_target,
            .optimize = .ReleaseSmall,
            .imports = &.{
                .{ .name = "platform", .module = platform_module_wasm },
                .{ .name = "imgui", .module = imgui_module_wasm },
            },
        }),
    });

    test_ui_zig.rdynamic = true;
    test_ui_zig.entry = .disabled;

    b.installArtifact(test_ui_zig);

    // ========================================================================
    // Desktop Apps (Native with SDL2)
    // ========================================================================
    const platform_module_native = b.createModule(.{
        .root_source_file = b.path("src/sdk/zig/platform.zig"),
        .target = native_target,
        .optimize = native_optimize,
    });

    const imgui_module_native = b.createModule(.{
        .root_source_file = b.path("src/sdk/zig/imgui.zig"),
        .target = native_target,
        .optimize = native_optimize,
        .imports = &.{
            .{ .name = "platform", .module = platform_module_native },
        },
    });

    // Circles app module (native)
    const circles_app_module = b.createModule(.{
        .root_source_file = b.path("src/apps/circles_zig/main.zig"),
        .target = native_target,
        .optimize = native_optimize,
        .imports = &.{
            .{ .name = "platform", .module = platform_module_native },
        },
    });

    // Desktop emulator for circles
    const circles_desktop = b.addExecutable(.{
        .name = "circles_desktop",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/ports/desktop/main.zig"),
            .target = native_target,
            .optimize = native_optimize,
            .imports = &.{
                .{ .name = "platform", .module = platform_module_native },
                .{ .name = "app", .module = circles_app_module },
            },
        }),
    });

    circles_desktop.linkSystemLibrary("SDL2");
    circles_desktop.linkLibC();

    b.installArtifact(circles_desktop);

    // Test UI app module (native)
    const test_ui_app_module = b.createModule(.{
        .root_source_file = b.path("src/apps/test_ui_zig/main.zig"),
        .target = native_target,
        .optimize = native_optimize,
        .imports = &.{
            .{ .name = "platform", .module = platform_module_native },
            .{ .name = "imgui", .module = imgui_module_native },
        },
    });

    // Desktop emulator for test_ui
    const test_ui_desktop = b.addExecutable(.{
        .name = "test_ui_desktop",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/ports/desktop/main.zig"),
            .target = native_target,
            .optimize = native_optimize,
            .imports = &.{
                .{ .name = "platform", .module = platform_module_native },
                .{ .name = "app", .module = test_ui_app_module },
            },
        }),
    });

    test_ui_desktop.linkSystemLibrary("SDL2");
    test_ui_desktop.linkLibC();

    b.installArtifact(test_ui_desktop);

    // ========================================================================
    // Build Steps
    // ========================================================================

    // WASM build step
    const wasm_step = b.step("wasm", "Build only WASM apps");
    wasm_step.dependOn(&b.addInstallArtifact(circles_zig, .{}).step);
    wasm_step.dependOn(&b.addInstallArtifact(test_ui_zig, .{}).step);

    // Web build step - build WASM apps and copy to www/
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

    // Desktop build step
    const desktop_step = b.step("desktop", "Build desktop emulator apps");
    desktop_step.dependOn(&b.addInstallArtifact(circles_desktop, .{}).step);
    desktop_step.dependOn(&b.addInstallArtifact(test_ui_desktop, .{}).step);

    // Run step for circles desktop
    const run_circles = b.addRunArtifact(circles_desktop);
    run_circles.step.dependOn(b.getInstallStep());
    const run_circles_step = b.step("run-circles", "Run circles app in desktop emulator");
    run_circles_step.dependOn(&run_circles.step);

    // Run step for test_ui desktop
    const run_test_ui = b.addRunArtifact(test_ui_desktop);
    run_test_ui.step.dependOn(b.getInstallStep());
    const run_test_ui_step = b.step("run-test-ui", "Run test_ui app in desktop emulator");
    run_test_ui_step.dependOn(&run_test_ui.step);
}
