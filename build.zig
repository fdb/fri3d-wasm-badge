const std = @import("std");

pub fn build(b: *std.Build) void {
    // ========================================================================
    // WASM Apps - All apps are compiled to WASM
    // ========================================================================
    // Apps are always WASM files. They run on:
    // - Web: loaded by browser via platform.js
    // - Desktop: loaded by emulator (C++ with WAMR, or future Zig runtime)
    // - ESP32: loaded by firmware runtime

    const zig_wasm_target = b.resolveTargetQuery(.{
        .cpu_arch = .wasm32,
        .os_tag = .freestanding,
    });

    // Shared SDK modules
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

    // ========================================================================
    // Circles App (WASM)
    // ========================================================================
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

    // ========================================================================
    // Test UI App (WASM)
    // ========================================================================
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
    // Desktop Emulator (Native)
    // ========================================================================
    // Requires: SDL2 (system), WAMR (pre-built via CMake)
    // Build WAMR first: cmake -B build/emulator && cmake --build build/emulator

    const native_target = b.standardTargetOptions(.{});
    const native_optimize = b.standardOptimizeOption(.{});

    const emulator = b.addExecutable(.{
        .name = "fri3d_emulator",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/ports/emulator/main.zig"),
            .target = native_target,
            .optimize = native_optimize,
        }),
    });

    // SDL2 (use pkg-config or fallback to Homebrew path)
    emulator.linkSystemLibrary("SDL2");
    emulator.addLibraryPath(.{ .cwd_relative = "/opt/homebrew/lib" });
    emulator.addIncludePath(.{ .cwd_relative = "/opt/homebrew/include" });
    emulator.linkLibC();

    // WAMR include paths
    emulator.addIncludePath(b.path("libs/wasm-micro-runtime/core/iwasm/include"));

    // Link against pre-built WAMR library (built via CMake)
    // Build first: cmake -B build/emulator && cmake --build build/emulator
    emulator.addLibraryPath(.{ .cwd_relative = "build/emulator/wamr" });
    emulator.linkSystemLibrary("iwasm");

    // Emulator build step
    const emulator_step = b.step("emulator", "Build desktop emulator (requires WAMR built via CMake)");
    const install_emulator = b.addInstallArtifact(emulator, .{});
    emulator_step.dependOn(&install_emulator.step);

    // Run emulator step
    const run_emulator = b.addRunArtifact(emulator);
    run_emulator.step.dependOn(&install_emulator.step);
    if (b.args) |args| {
        run_emulator.addArgs(args);
    }
    const run_step = b.step("run", "Run the emulator (pass WASM file as argument)");
    run_step.dependOn(&run_emulator.step);

    // ========================================================================
    // Build Steps
    // ========================================================================

    // Default: build all WASM apps
    b.getInstallStep().dependOn(&b.addInstallArtifact(circles_zig, .{}).step);
    b.getInstallStep().dependOn(&b.addInstallArtifact(test_ui_zig, .{}).step);

    // Web step: build WASM apps and copy to www/
    const web_step = b.step("web", "Build web version (WASM apps + platform files)");

    const copy_html = b.addInstallFile(b.path("src/ports/web/index.html"), "../www/index.html");
    const copy_js = b.addInstallFile(b.path("src/ports/web/platform.js"), "../www/platform.js");
    const copy_circles = b.addInstallFile(circles_zig.getEmittedBin(), "../www/circles_zig.wasm");
    const copy_test_ui = b.addInstallFile(test_ui_zig.getEmittedBin(), "../www/test_ui_zig.wasm");

    web_step.dependOn(&copy_html.step);
    web_step.dependOn(&copy_js.step);
    web_step.dependOn(&copy_circles.step);
    web_step.dependOn(&copy_test_ui.step);
}
