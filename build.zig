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

    // Shared runtime modules
    const platform_module = b.createModule(.{
        .root_source_file = b.path("src/runtime/platform.zig"),
        .target = zig_wasm_target,
        .optimize = .ReleaseSmall,
    });

    const imgui_module = b.createModule(.{
        .root_source_file = b.path("src/runtime/imgui.zig"),
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
    // Builds WAMR from source and links with SDL2

    const native_target = b.standardTargetOptions(.{});
    const native_optimize = b.standardOptimizeOption(.{});

    // Build WAMR library from source
    const wamr = buildWamr(b, native_target, native_optimize);

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

    // Link WAMR
    emulator.linkLibrary(wamr);
    emulator.addIncludePath(b.path("libs/wasm-micro-runtime/core/iwasm/include"));

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

// ============================================================================
// WAMR Build (WebAssembly Micro Runtime)
// ============================================================================
// Builds WAMR interpreter-only from source (no AOT, no JIT)

fn buildWamr(b: *std.Build, target: std.Build.ResolvedTarget, optimize: std.builtin.OptimizeMode) *std.Build.Step.Compile {
    const wamr = b.addLibrary(.{
        .linkage = .static,
        .name = "iwasm",
        .root_module = b.createModule(.{
            .root_source_file = null,
            .target = target,
            .optimize = optimize,
        }),
    });

    const wamr_root = "libs/wasm-micro-runtime";

    // Platform detection
    const os_tag = target.result.os.tag;
    const cpu_arch = target.result.cpu.arch;

    // Platform define
    const platform_define: []const u8 = if (os_tag == .macos)
        "-DBH_PLATFORM_DARWIN"
    else if (os_tag == .linux)
        "-DBH_PLATFORM_LINUX"
    else
        "-DBH_PLATFORM_LINUX"; // fallback

    // C flags
    const flags: []const []const u8 = &.{
        "-std=gnu99",
        "-ffunction-sections",
        "-fdata-sections",
        "-Wall",
        "-Wno-unused-parameter",
        "-Wno-pedantic",
        // WAMR configuration defines
        "-DBH_MALLOC=wasm_runtime_malloc",
        "-DBH_FREE=wasm_runtime_free",
        "-DWASM_ENABLE_INTERP=1",
        "-DWASM_ENABLE_FAST_INTERP=1",
        "-DWASM_ENABLE_LIBC_BUILTIN=1",
        "-DWASM_ENABLE_BULK_MEMORY=1",
        "-DWASM_ENABLE_BULK_MEMORY_OPT=1",
        "-DWASM_ENABLE_SHARED_MEMORY=0",
        "-DWASM_ENABLE_THREAD_MGR=0",
        "-DWASM_ENABLE_TAIL_CALL=0",
        "-DWASM_ENABLE_SIMD=0",
        "-DWASM_ENABLE_REF_TYPES=0",
        "-DWASM_ENABLE_MULTI_MODULE=0",
        "-DWASM_ENABLE_MEMORY_PROFILING=0",
        "-DWASM_ENABLE_LOAD_CUSTOM_SECTION=0",
        "-DWASM_ENABLE_CUSTOM_NAME_SECTION=0",
        "-DWASM_ENABLE_DUMP_CALL_STACK=0",
        "-DWASM_ENABLE_AOT=0",
        "-DWASM_ENABLE_WAMR_COMPILER=0",
        "-DWASM_ENABLE_LIBC_WASI=0",
        "-DWASM_ENABLE_DEBUG_INTERP=0",
        "-DWASM_HAVE_MREMAP=0",
        platform_define,
    };

    // Include paths
    wamr.addIncludePath(b.path(wamr_root ++ "/core/iwasm/include"));
    wamr.addIncludePath(b.path(wamr_root ++ "/core/iwasm/common"));
    wamr.addIncludePath(b.path(wamr_root ++ "/core/iwasm/interpreter"));
    wamr.addIncludePath(b.path(wamr_root ++ "/core/iwasm/libraries/libc-builtin"));
    wamr.addIncludePath(b.path(wamr_root ++ "/core/shared/include"));
    wamr.addIncludePath(b.path(wamr_root ++ "/core/shared/platform/include"));
    wamr.addIncludePath(b.path(wamr_root ++ "/core/shared/utils"));
    wamr.addIncludePath(b.path(wamr_root ++ "/core/shared/mem-alloc"));

    // Platform-specific includes
    if (os_tag == .macos) {
        wamr.addIncludePath(b.path(wamr_root ++ "/core/shared/platform/darwin"));
    } else if (os_tag == .linux) {
        wamr.addIncludePath(b.path(wamr_root ++ "/core/shared/platform/linux"));
    }

    // ---- Source files ----

    // Interpreter sources
    wamr.addCSourceFiles(.{
        .root = b.path(wamr_root),
        .files = &.{
            "core/iwasm/interpreter/wasm_loader.c",
            "core/iwasm/interpreter/wasm_runtime.c",
            "core/iwasm/interpreter/wasm_interp_fast.c",
        },
        .flags = flags,
    });

    // Common sources
    wamr.addCSourceFiles(.{
        .root = b.path(wamr_root),
        .files = &.{
            "core/iwasm/common/wasm_application.c",
            "core/iwasm/common/wasm_blocking_op.c",
            "core/iwasm/common/wasm_c_api.c",
            "core/iwasm/common/wasm_exec_env.c",
            "core/iwasm/common/wasm_loader_common.c",
            "core/iwasm/common/wasm_memory.c",
            "core/iwasm/common/wasm_native.c",
            "core/iwasm/common/wasm_runtime_common.c",
            "core/iwasm/common/wasm_shared_memory.c",
        },
        .flags = flags,
    });

    // Libc-builtin
    wamr.addCSourceFiles(.{
        .root = b.path(wamr_root),
        .files = &.{
            "core/iwasm/libraries/libc-builtin/libc_builtin_wrapper.c",
        },
        .flags = flags,
    });

    // Shared utils
    wamr.addCSourceFiles(.{
        .root = b.path(wamr_root),
        .files = &.{
            "core/shared/utils/bh_assert.c",
            "core/shared/utils/bh_bitmap.c",
            "core/shared/utils/bh_common.c",
            "core/shared/utils/bh_hashmap.c",
            "core/shared/utils/bh_leb128.c",
            "core/shared/utils/bh_list.c",
            "core/shared/utils/bh_log.c",
            "core/shared/utils/bh_queue.c",
            "core/shared/utils/bh_vector.c",
            "core/shared/utils/runtime_timer.c",
        },
        .flags = flags,
    });

    // Memory allocator
    wamr.addCSourceFiles(.{
        .root = b.path(wamr_root),
        .files = &.{
            "core/shared/mem-alloc/mem_alloc.c",
            "core/shared/mem-alloc/ems/ems_alloc.c",
            "core/shared/mem-alloc/ems/ems_gc.c",
            "core/shared/mem-alloc/ems/ems_hmu.c",
            "core/shared/mem-alloc/ems/ems_kfc.c",
        },
        .flags = flags,
    });

    // Platform sources (POSIX common)
    wamr.addCSourceFiles(.{
        .root = b.path(wamr_root),
        .files = &.{
            "core/shared/platform/common/posix/posix_blocking_op.c",
            "core/shared/platform/common/posix/posix_malloc.c",
            "core/shared/platform/common/posix/posix_memmap.c",
            "core/shared/platform/common/posix/posix_sleep.c",
            "core/shared/platform/common/posix/posix_thread.c",
            "core/shared/platform/common/posix/posix_time.c",
            "core/shared/platform/common/memory/mremap.c",
        },
        .flags = flags,
    });

    // Platform-specific init
    if (os_tag == .macos) {
        wamr.addCSourceFiles(.{
            .root = b.path(wamr_root),
            .files = &.{"core/shared/platform/darwin/platform_init.c"},
            .flags = flags,
        });
    } else if (os_tag == .linux) {
        wamr.addCSourceFiles(.{
            .root = b.path(wamr_root),
            .files = &.{"core/shared/platform/linux/platform_init.c"},
            .flags = flags,
        });
    }

    // Architecture-specific assembly for invokeNative
    // On macOS, use the universal file which handles both arm64 and x86_64
    // On Linux, use architecture-specific files
    if (os_tag == .macos) {
        // Use osx_universal.s which auto-selects the right arch
        wamr.addCSourceFiles(.{
            .root = b.path(wamr_root),
            .files = &.{"core/iwasm/common/arch/invokeNative_osx_universal.s"},
            .flags = flags,
        });
    } else if (cpu_arch == .aarch64) {
        wamr.addCSourceFiles(.{
            .root = b.path(wamr_root),
            .files = &.{"core/iwasm/common/arch/invokeNative_aarch64.s"},
            .flags = flags,
        });
    } else if (cpu_arch == .x86_64) {
        wamr.addCSourceFiles(.{
            .root = b.path(wamr_root),
            .files = &.{"core/iwasm/common/arch/invokeNative_em64.s"},
            .flags = flags,
        });
    } else {
        // Fallback to general C implementation (limited to 20 args)
        wamr.addCSourceFiles(.{
            .root = b.path(wamr_root),
            .files = &.{"core/iwasm/common/arch/invokeNative_general.c"},
            .flags = flags,
        });
    }

    wamr.linkLibC();

    return wamr;
}
