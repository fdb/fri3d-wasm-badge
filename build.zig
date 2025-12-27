const std = @import("std");

pub fn build(b: *std.Build) void {
    // Standard target options - defaults to native
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // ========================================================================
    // u8g2 Graphics Library (compiled from C source)
    // ========================================================================
    const u8g2 = b.addLibrary(.{
        .name = "u8g2",
        .linkage = .static,
        .root_module = b.createModule(.{
            .target = target,
            .optimize = optimize,
            .link_libc = true,
        }),
    });

    // Add all u8g2 C source files (129 files for all display drivers)
    const u8g2_sources = [_][]const u8{
        "libs/u8g2/csrc/mui.c",
        "libs/u8g2/csrc/mui_u8g2.c",
        "libs/u8g2/csrc/u8g2_arc.c",
        "libs/u8g2/csrc/u8g2_bitmap.c",
        "libs/u8g2/csrc/u8g2_box.c",
        "libs/u8g2/csrc/u8g2_buffer.c",
        "libs/u8g2/csrc/u8g2_button.c",
        "libs/u8g2/csrc/u8g2_circle.c",
        "libs/u8g2/csrc/u8g2_cleardisplay.c",
        "libs/u8g2/csrc/u8g2_d_memory.c",
        "libs/u8g2/csrc/u8g2_d_setup.c",
        "libs/u8g2/csrc/u8g2_font.c",
        "libs/u8g2/csrc/u8g2_fonts.c",
        "libs/u8g2/csrc/u8g2_hvline.c",
        "libs/u8g2/csrc/u8g2_input_value.c",
        "libs/u8g2/csrc/u8g2_intersection.c",
        "libs/u8g2/csrc/u8g2_kerning.c",
        "libs/u8g2/csrc/u8g2_line.c",
        "libs/u8g2/csrc/u8g2_ll_hvline.c",
        "libs/u8g2/csrc/u8g2_message.c",
        "libs/u8g2/csrc/u8g2_polygon.c",
        "libs/u8g2/csrc/u8g2_selection_list.c",
        "libs/u8g2/csrc/u8g2_setup.c",
        "libs/u8g2/csrc/u8log.c",
        "libs/u8g2/csrc/u8log_u8g2.c",
        "libs/u8g2/csrc/u8log_u8x8.c",
        "libs/u8g2/csrc/u8x8_8x8.c",
        "libs/u8g2/csrc/u8x8_byte.c",
        "libs/u8g2/csrc/u8x8_cad.c",
        "libs/u8g2/csrc/u8x8_capture.c",
        "libs/u8g2/csrc/u8x8_d_a2printer.c",
        "libs/u8g2/csrc/u8x8_d_ch1120.c",
        "libs/u8g2/csrc/u8x8_d_gp1247ai.c",
        "libs/u8g2/csrc/u8x8_d_gp1287ai.c",
        "libs/u8g2/csrc/u8x8_d_gp1294ai.c",
        "libs/u8g2/csrc/u8x8_d_gu800.c",
        "libs/u8g2/csrc/u8x8_d_hd44102.c",
        "libs/u8g2/csrc/u8x8_d_il3820_296x128.c",
        "libs/u8g2/csrc/u8x8_d_ist3020.c",
        "libs/u8g2/csrc/u8x8_d_ist3088.c",
        "libs/u8g2/csrc/u8x8_d_ist7920.c",
        "libs/u8g2/csrc/u8x8_d_ks0108.c",
        "libs/u8g2/csrc/u8x8_d_lc7981.c",
        "libs/u8g2/csrc/u8x8_d_ld7032_60x32.c",
        "libs/u8g2/csrc/u8x8_d_ls013b7dh03.c",
        "libs/u8g2/csrc/u8x8_d_max7219.c",
        "libs/u8g2/csrc/u8x8_d_pcd8544_84x48.c",
        "libs/u8g2/csrc/u8x8_d_pcf8812.c",
        "libs/u8g2/csrc/u8x8_d_pcf8814_hx1230.c",
        "libs/u8g2/csrc/u8x8_d_s1d15300.c",
        "libs/u8g2/csrc/u8x8_d_s1d15721.c",
        "libs/u8g2/csrc/u8x8_d_s1d15e06.c",
        "libs/u8g2/csrc/u8x8_d_sbn1661.c",
        "libs/u8g2/csrc/u8x8_d_sed1330.c",
        "libs/u8g2/csrc/u8x8_d_sh1106_64x32.c",
        "libs/u8g2/csrc/u8x8_d_sh1106_72x40.c",
        "libs/u8g2/csrc/u8x8_d_sh1107.c",
        "libs/u8g2/csrc/u8x8_d_sh1108.c",
        "libs/u8g2/csrc/u8x8_d_sh1122.c",
        "libs/u8g2/csrc/u8x8_d_ssd1305.c",
        "libs/u8g2/csrc/u8x8_d_ssd1306_128x32.c",
        "libs/u8g2/csrc/u8x8_d_ssd1306_128x64_noname.c",
        "libs/u8g2/csrc/u8x8_d_ssd1306_2040x16.c",
        "libs/u8g2/csrc/u8x8_d_ssd1306_48x64.c",
        "libs/u8g2/csrc/u8x8_d_ssd1306_64x32.c",
        "libs/u8g2/csrc/u8x8_d_ssd1306_64x48.c",
        "libs/u8g2/csrc/u8x8_d_ssd1306_72x40.c",
        "libs/u8g2/csrc/u8x8_d_ssd1306_96x16.c",
        "libs/u8g2/csrc/u8x8_d_ssd1306_96x40.c",
        "libs/u8g2/csrc/u8x8_d_ssd1309.c",
        "libs/u8g2/csrc/u8x8_d_ssd1312.c",
        "libs/u8g2/csrc/u8x8_d_ssd1315_128x64_noname.c",
        "libs/u8g2/csrc/u8x8_d_ssd1316.c",
        "libs/u8g2/csrc/u8x8_d_ssd1317.c",
        "libs/u8g2/csrc/u8x8_d_ssd1318.c",
        "libs/u8g2/csrc/u8x8_d_ssd1320.c",
        "libs/u8g2/csrc/u8x8_d_ssd1322.c",
        "libs/u8g2/csrc/u8x8_d_ssd1325.c",
        "libs/u8g2/csrc/u8x8_d_ssd1326.c",
        "libs/u8g2/csrc/u8x8_d_ssd1327.c",
        "libs/u8g2/csrc/u8x8_d_ssd1329.c",
        "libs/u8g2/csrc/u8x8_d_ssd1362.c",
        "libs/u8g2/csrc/u8x8_d_ssd1363.c",
        "libs/u8g2/csrc/u8x8_d_ssd1606_172x72.c",
        "libs/u8g2/csrc/u8x8_d_ssd1607_200x200.c",
        "libs/u8g2/csrc/u8x8_d_st7302.c",
        "libs/u8g2/csrc/u8x8_d_st7305.c",
        "libs/u8g2/csrc/u8x8_d_st7511.c",
        "libs/u8g2/csrc/u8x8_d_st75160.c",
        "libs/u8g2/csrc/u8x8_d_st75161.c",
        "libs/u8g2/csrc/u8x8_d_st75256.c",
        "libs/u8g2/csrc/u8x8_d_st7528.c",
        "libs/u8g2/csrc/u8x8_d_st75320.c",
        "libs/u8g2/csrc/u8x8_d_st7539.c",
        "libs/u8g2/csrc/u8x8_d_st7565.c",
        "libs/u8g2/csrc/u8x8_d_st7567.c",
        "libs/u8g2/csrc/u8x8_d_st7571.c",
        "libs/u8g2/csrc/u8x8_d_st7586s_erc240160.c",
        "libs/u8g2/csrc/u8x8_d_st7586s_jlx320160.c",
        "libs/u8g2/csrc/u8x8_d_st7586s_jlx384160.c",
        "libs/u8g2/csrc/u8x8_d_st7586s_md240128.c",
        "libs/u8g2/csrc/u8x8_d_st7586s_s028hn118a.c",
        "libs/u8g2/csrc/u8x8_d_st7586s_ymc240160.c",
        "libs/u8g2/csrc/u8x8_d_st7588.c",
        "libs/u8g2/csrc/u8x8_d_st7920.c",
        "libs/u8g2/csrc/u8x8_d_stdio.c",
        "libs/u8g2/csrc/u8x8_d_t6963.c",
        "libs/u8g2/csrc/u8x8_d_uc1601.c",
        "libs/u8g2/csrc/u8x8_d_uc1604.c",
        "libs/u8g2/csrc/u8x8_d_uc1608.c",
        "libs/u8g2/csrc/u8x8_d_uc1609.c",
        "libs/u8g2/csrc/u8x8_d_uc1610.c",
        "libs/u8g2/csrc/u8x8_d_uc1611.c",
        "libs/u8g2/csrc/u8x8_d_uc1617.c",
        "libs/u8g2/csrc/u8x8_d_uc1628.c",
        "libs/u8g2/csrc/u8x8_d_uc1638.c",
        "libs/u8g2/csrc/u8x8_d_uc1701_dogs102.c",
        "libs/u8g2/csrc/u8x8_d_uc1701_mini12864.c",
        "libs/u8g2/csrc/u8x8_debounce.c",
        "libs/u8g2/csrc/u8x8_display.c",
        "libs/u8g2/csrc/u8x8_fonts.c",
        "libs/u8g2/csrc/u8x8_gpio.c",
        "libs/u8g2/csrc/u8x8_input_value.c",
        "libs/u8g2/csrc/u8x8_message.c",
        "libs/u8g2/csrc/u8x8_selection_list.c",
        "libs/u8g2/csrc/u8x8_setup.c",
        "libs/u8g2/csrc/u8x8_string.c",
        "libs/u8g2/csrc/u8x8_u16toa.c",
        "libs/u8g2/csrc/u8x8_u8toa.c",
    };

    u8g2.root_module.addCSourceFiles(.{
        .files = &u8g2_sources,
        .flags = &.{"-std=c99"},
    });
    u8g2.root_module.addIncludePath(b.path("libs/u8g2/csrc"));

    // ========================================================================
    // WAMR (WebAssembly Micro Runtime) - compiled from C source
    // ========================================================================
    const wamr = b.addLibrary(.{
        .name = "vmlib",
        .linkage = .static,
        .root_module = b.createModule(.{
            .target = target,
            .optimize = optimize,
            .link_libc = true,
        }),
    });

    // WAMR configuration defines
    const wamr_defines = [_][]const u8{
        "-DWASM_ENABLE_INTERP=1",
        "-DWASM_ENABLE_FAST_INTERP=1",
        "-DWASM_ENABLE_LIBC_BUILTIN=1",
        "-DWASM_ENABLE_LIBC_WASI=0",
        "-DWASM_ENABLE_AOT=0",
        "-DWASM_ENABLE_JIT=0",
        "-DWASM_ENABLE_MULTI_MODULE=0",
        "-DWASM_ENABLE_BULK_MEMORY=0",
        "-DWASM_ENABLE_REF_TYPES=0",
        "-DWASM_ENABLE_SIMD=0",
        "-DWASM_ENABLE_THREAD_MGR=0",
        "-DWASM_ENABLE_MEMORY_PROFILING=0",
        "-DWASM_ENABLE_MEMORY_TRACING=0",
        "-DWASM_ENABLE_PERF_PROFILING=0",
        "-DWASM_ENABLE_DUMP_CALL_STACK=0",
        "-DWASM_ENABLE_CUSTOM_NAME_SECTION=0",
        "-DWASM_ENABLE_TAIL_CALL=0",
        "-DWASM_ENABLE_SHARED_MEMORY=0",
        "-DWASM_ENABLE_MINI_LOADER=0",
        "-DWASM_ENABLE_SPEC_TEST=0",
        "-DWASM_ENABLE_MODULE_INST_CONTEXT=0",
        "-DWASM_ENABLE_EXCE_HANDLING=0",
        "-DWASM_ENABLE_TAGS=0",
        "-DWASM_ENABLE_GC=0",
        "-DWASM_ENABLE_STRINGREF=0",
        "-DWASM_ENABLE_QUICK_AOT_ENTRY=0",
        "-DWASM_ENABLE_STATIC_PGO=0",
        "-DWASM_ENABLE_WAMR_COMPILER=0",
        "-DWASM_ENABLE_LOAD_CUSTOM_SECTION=0",
        "-DWASM_CONFIGURABLE_BOUNDS_CHECKS=0",
        "-DWASM_ENABLE_GLOBAL_HEAP_POOL=0",
        "-DWASM_MEM_ALLOC_WITH_USAGE=0",
        "-DWASM_CPU_SUPPORTS_UNALIGNED_ADDR_ACCESS=1",
        "-DWASM_ENABLE_HASH_BLOCK_ADDR=0",
        "-DWASM_DISABLE_WRITE_GS_BASE=1",
        "-DBH_MALLOC=wasm_runtime_malloc",
        "-DBH_FREE=wasm_runtime_free",
        "-DBH_PLATFORM_LINUX",
        "-DBUILD_TARGET_X86_64",
    };

    // Core WAMR sources
    const wamr_sources = [_][]const u8{
        "libs/wasm-micro-runtime/core/iwasm/interpreter/wasm_loader.c",
        "libs/wasm-micro-runtime/core/iwasm/interpreter/wasm_runtime.c",
        "libs/wasm-micro-runtime/core/iwasm/interpreter/wasm_interp_fast.c",
        "libs/wasm-micro-runtime/core/iwasm/common/wasm_runtime_common.c",
        "libs/wasm-micro-runtime/core/iwasm/common/wasm_native.c",
        "libs/wasm-micro-runtime/core/iwasm/common/wasm_exec_env.c",
        "libs/wasm-micro-runtime/core/iwasm/common/wasm_memory.c",
        "libs/wasm-micro-runtime/core/iwasm/common/wasm_c_api.c",
        "libs/wasm-micro-runtime/core/iwasm/common/wasm_application.c",
        "libs/wasm-micro-runtime/core/iwasm/common/wasm_loader_common.c",
        "libs/wasm-micro-runtime/core/iwasm/common/wasm_blocking_op.c",
        "libs/wasm-micro-runtime/core/iwasm/common/arch/invokeNative_general.c",
        "libs/wasm-micro-runtime/core/iwasm/libraries/libc-builtin/libc_builtin_wrapper.c",
        "libs/wasm-micro-runtime/core/shared/utils/bh_assert.c",
        "libs/wasm-micro-runtime/core/shared/utils/bh_common.c",
        "libs/wasm-micro-runtime/core/shared/utils/bh_hashmap.c",
        "libs/wasm-micro-runtime/core/shared/utils/bh_leb128.c",
        "libs/wasm-micro-runtime/core/shared/utils/bh_list.c",
        "libs/wasm-micro-runtime/core/shared/utils/bh_log.c",
        "libs/wasm-micro-runtime/core/shared/utils/bh_queue.c",
        "libs/wasm-micro-runtime/core/shared/utils/bh_vector.c",
        "libs/wasm-micro-runtime/core/shared/utils/runtime_timer.c",
        "libs/wasm-micro-runtime/core/shared/mem-alloc/mem_alloc.c",
        "libs/wasm-micro-runtime/core/shared/mem-alloc/ems/ems_alloc.c",
        "libs/wasm-micro-runtime/core/shared/mem-alloc/ems/ems_hmu.c",
        "libs/wasm-micro-runtime/core/shared/mem-alloc/ems/ems_kfc.c",
        "libs/wasm-micro-runtime/core/shared/platform/linux/platform_init.c",
        "libs/wasm-micro-runtime/core/shared/platform/common/posix/posix_thread.c",
        "libs/wasm-micro-runtime/core/shared/platform/common/posix/posix_time.c",
        "libs/wasm-micro-runtime/core/shared/platform/common/posix/posix_malloc.c",
        "libs/wasm-micro-runtime/core/shared/platform/common/posix/posix_memmap.c",
        "libs/wasm-micro-runtime/core/shared/platform/common/posix/posix_blocking_op.c",
        "libs/wasm-micro-runtime/core/shared/platform/common/memory/mremap.c",
    };

    wamr.root_module.addCSourceFiles(.{
        .files = &wamr_sources,
        .flags = &wamr_defines,
    });

    // WAMR include paths
    wamr.root_module.addIncludePath(b.path("libs/wasm-micro-runtime/core/iwasm/include"));
    wamr.root_module.addIncludePath(b.path("libs/wasm-micro-runtime/core/iwasm/interpreter"));
    wamr.root_module.addIncludePath(b.path("libs/wasm-micro-runtime/core/iwasm/common"));
    wamr.root_module.addIncludePath(b.path("libs/wasm-micro-runtime/core/iwasm/libraries/libc-builtin"));
    wamr.root_module.addIncludePath(b.path("libs/wasm-micro-runtime/core/shared/include"));
    wamr.root_module.addIncludePath(b.path("libs/wasm-micro-runtime/core/shared/utils"));
    wamr.root_module.addIncludePath(b.path("libs/wasm-micro-runtime/core/shared/utils/uncommon"));
    wamr.root_module.addIncludePath(b.path("libs/wasm-micro-runtime/core/shared/mem-alloc"));
    wamr.root_module.addIncludePath(b.path("libs/wasm-micro-runtime/core/shared/mem-alloc/ems"));
    wamr.root_module.addIncludePath(b.path("libs/wasm-micro-runtime/core/shared/platform/include"));
    wamr.root_module.addIncludePath(b.path("libs/wasm-micro-runtime/core/shared/platform/linux"));

    // ========================================================================
    // lodepng (for screenshots)
    // ========================================================================
    const lodepng = b.addLibrary(.{
        .name = "lodepng",
        .linkage = .static,
        .root_module = b.createModule(.{
            .target = target,
            .optimize = optimize,
            .link_libcpp = true,
        }),
    });
    lodepng.root_module.addCSourceFiles(.{
        .files = &.{"libs/lodepng/lodepng.cpp"},
        .flags = &.{"-std=c++14"},
    });
    lodepng.root_module.addIncludePath(b.path("libs/lodepng"));

    // ========================================================================
    // Desktop Emulator (SDL)
    // ========================================================================
    const emulator = b.addExecutable(.{
        .name = "fri3d_emulator",
        .target = target,
        .optimize = optimize,
    });

    emulator.root_module.addCSourceFiles(.{
        .files = &.{
            "src/emulator/main.cpp",
            "src/emulator/display_sdl.cpp",
            "src/emulator/input_sdl.cpp",
            "src/runtime/canvas.cpp",
            "src/runtime/input.cpp",
            "src/runtime/random.cpp",
            "src/runtime/wasm_runner.cpp",
            "src/runtime/app_manager.cpp",
        },
        .flags = &.{
            "-std=c++14",
            "-DBH_PLATFORM_LINUX",
        },
    });

    // Include paths for emulator
    emulator.root_module.addIncludePath(b.path("src/runtime"));
    emulator.root_module.addIncludePath(b.path("src/emulator"));
    emulator.root_module.addIncludePath(b.path("libs/u8g2/csrc"));
    emulator.root_module.addIncludePath(b.path("libs/lodepng"));
    emulator.root_module.addIncludePath(b.path("libs/wasm-micro-runtime/core/iwasm/include"));

    // Link libraries
    emulator.linkLibrary(u8g2);
    emulator.linkLibrary(wamr);
    emulator.linkLibrary(lodepng);
    emulator.linkSystemLibrary("SDL2");
    emulator.root_module.link_libcpp = true;
    emulator.root_module.link_libc = true;

    b.installArtifact(emulator);

    // ========================================================================
    // WASM Apps (using WASI for libc support)
    // ========================================================================
    const wasm_target = b.resolveTargetQuery(.{
        .cpu_arch = .wasm32,
        .os_tag = .wasi,
    });

    const apps = [_][]const u8{ "circles", "mandelbrot", "test_drawing", "test_ui" };

    for (apps) |app_name| {
        const app = b.addExecutable(.{
            .name = app_name,
            .target = wasm_target,
            .optimize = .ReleaseSmall,
        });

        app.root_module.link_libc = true;

        // For now, compile the C version of apps
        const main_path = b.fmt("src/apps/{s}/main.c", .{app_name});
        app.root_module.addCSourceFiles(.{
            .files = &.{main_path},
            .flags = &.{ "-O3", "-D__wasi__" },
        });

        // IMGUI library (compiled into each app)
        app.root_module.addCSourceFiles(.{
            .files = &.{"src/sdk/imgui.c"},
            .flags = &.{ "-O3", "-D__wasi__" },
        });

        app.root_module.addIncludePath(b.path("src/sdk"));

        // Export functions for WASM
        app.rdynamic = true;
        app.entry = .disabled;

        // Install to zig-out/bin/
        b.installArtifact(app);
    }

    // ========================================================================
    // Pure Zig WASM Apps
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
        .root_source_file = b.path("src/apps/circles_zig/main.zig"),
        .target = zig_wasm_target,
        .optimize = .ReleaseSmall,
    });

    circles_zig.root_module.addImport("platform", platform_module);
    circles_zig.rdynamic = true;
    circles_zig.entry = .disabled;

    b.installArtifact(circles_zig);

    // Test UI app in pure Zig (uses IMGUI)
    const test_ui_zig = b.addExecutable(.{
        .name = "test_ui_zig",
        .root_source_file = b.path("src/apps/test_ui_zig/main.zig"),
        .target = zig_wasm_target,
        .optimize = .ReleaseSmall,
    });

    test_ui_zig.root_module.addImport("platform", platform_module);
    test_ui_zig.root_module.addImport("imgui", imgui_module);
    test_ui_zig.rdynamic = true;
    test_ui_zig.entry = .disabled;

    b.installArtifact(test_ui_zig);

    // ========================================================================
    // Run command
    // ========================================================================
    const run_cmd = b.addRunArtifact(emulator);
    run_cmd.step.dependOn(b.getInstallStep());

    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const run_step = b.step("run", "Run the desktop emulator");
    run_step.dependOn(&run_cmd.step);

    // ========================================================================
    // Test step (run with Zig circles app)
    // ========================================================================
    const test_cmd = b.addRunArtifact(emulator);
    test_cmd.step.dependOn(b.getInstallStep());
    test_cmd.addArgs(&.{ "--test", "zig-out/bin/circles_zig.wasm" });

    const test_step = b.step("test", "Run emulator with Zig circles app");
    test_step.dependOn(&test_cmd.step);

    // ========================================================================
    // Web build step - copy web files and WASM apps to www/
    // ========================================================================
    const web_step = b.step("web", "Build web version (copy files to www/)");

    // Copy web platform files
    const copy_html = b.addInstallFile(b.path("src/ports/web/index.html"), "../www/index.html");
    const copy_js = b.addInstallFile(b.path("src/ports/web/platform.js"), "../www/platform.js");

    web_step.dependOn(&copy_html.step);
    web_step.dependOn(&copy_js.step);

    // Copy all WASM apps
    const wasm_apps_web = [_][]const u8{ "circles", "mandelbrot", "test_drawing", "test_ui", "circles_zig", "test_ui_zig" };
    for (wasm_apps_web) |app_name| {
        const dest_path = b.fmt("../www/{s}.wasm", .{app_name});
        const copy_wasm = b.addInstallFile(b.path(b.fmt("zig-out/bin/{s}.wasm", .{app_name})), dest_path);
        web_step.dependOn(&copy_wasm.step);
    }

    // Make web depend on building the apps first
    web_step.dependOn(b.getInstallStep());
}
