//! Desktop host.
//!
//! Interactive:  `cargo run -p fri3d-host-desktop --release`
//! Headless:     `fri3d --headless --app snake --keys ok,down --frames 3 --screenshot out.png`
//!
//! Keys: arrows / WASD = d-pad, Z / Enter = OK, X / Backspace = Back,
//! M / Escape = Menu (home), F12 = screenshot.

use fri3d_kernel::settings::IMAGE_LEN;
use fri3d_kernel::types::InputKey;
use fri3d_kernel::{Kernel, FRAMEBUFFER_LEN, SCREEN_HEIGHT, SCREEN_WIDTH};
use minifb::{Key, Scale, Window, WindowOptions};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

// Flipper-Zero-like amber backlight with black pixels.
const BG: u32 = 0x00FF8200;
const FG: u32 = 0x00000000;

struct Args {
    headless: bool,
    app: Option<String>,
    scene: Option<u32>,
    keys: Vec<String>,
    frames: u32,
    screenshot: Option<PathBuf>,
    apps_dir: Option<PathBuf>,
    seed: u32,
    list: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        headless: false,
        app: None,
        scene: None,
        keys: Vec::new(),
        frames: 1,
        screenshot: None,
        apps_dir: None,
        seed: 42,
        list: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        let mut val = || it.next().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--headless" => a.headless = true,
            "--list" => a.list = true,
            "--app" => a.app = Some(val()?),
            "--scene" => a.scene = Some(val()?.parse().map_err(|e| format!("--scene: {e}"))?),
            "--keys" => a.keys = val()?.split(',').map(str::to_string).collect(),
            "--frames" => a.frames = val()?.parse().map_err(|e| format!("--frames: {e}"))?,
            "--screenshot" => a.screenshot = Some(val()?.into()),
            "--apps-dir" => a.apps_dir = Some(val()?.into()),
            "--seed" => a.seed = val()?.parse().map_err(|e| format!("--seed: {e}"))?,
            "-h" | "--help" => {
                println!("fri3d [--headless] [--app ID] [--scene N] [--keys k1,k2] [--frames N] [--screenshot out.png] [--apps-dir DIR] [--seed S] [--list]");
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    Ok(a)
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("fri3d: {e}");
            std::process::exit(2);
        }
    };

    let mut kernel = Kernel::new();
    if let Err(e) = register_bundles(&mut kernel, args.apps_dir.as_deref()) {
        eprintln!("fri3d: {e}");
        std::process::exit(1);
    }
    if args.list {
        for i in 0..kernel.app_count() {
            println!("{i:>2}  {}", kernel.app_name(i).unwrap_or("?"));
        }
        return;
    }

    let settings_path = settings_path();
    if let Ok(img) = std::fs::read(&settings_path) {
        kernel.load_settings(&img);
    }

    kernel.random_seed(args.seed);
    let start = Instant::now();
    let now = || start.elapsed().as_millis() as u32;
    kernel.boot(now());

    if let Some(id) = &args.app {
        let idx = find_app(&kernel, id).unwrap_or_else(|| {
            eprintln!("fri3d: no app '{id}' (try --list)");
            std::process::exit(1);
        });
        kernel.start_app(idx);
    }
    if let Some(scene) = args.scene {
        kernel.set_scene(scene);
    }

    if args.headless {
        run_headless(&mut kernel, &args);
        return;
    }
    run_window(&mut kernel, &settings_path, start);
}

fn register_bundles(kernel: &mut Kernel, apps_dir: Option<&Path>) -> Result<(), String> {
    match apps_dir {
        None => {
            kernel
                .set_launcher(fri3d_apps::LAUNCHER)
                .map_err(|e| format!("launcher: {e:?}"))?;
            for app in fri3d_apps::APPS {
                kernel.add_app(app).map_err(|e| format!("app: {e:?}"))?;
            }
        }
        Some(dir) => {
            let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
                .map_err(|e| format!("{}: {e}", dir.display()))?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "fab"))
                .collect();
            paths.sort();
            for p in paths {
                // Bundles live for the whole process; leaking is the honest
                // way to hand the kernel a 'static slice.
                let bytes: &'static [u8] =
                    Box::leak(std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?.into_boxed_slice());
                if p.file_stem().is_some_and(|s| s == "launcher") {
                    kernel.set_launcher(bytes).map_err(|e| format!("{}: {e:?}", p.display()))?;
                } else {
                    kernel.add_app(bytes).map_err(|e| format!("{}: {e:?}", p.display()))?;
                }
            }
        }
    }
    Ok(())
}

fn find_app(kernel: &Kernel, id: &str) -> Option<usize> {
    (0..kernel.app_count()).find(|&i| {
        kernel
            .app_name(i)
            .is_some_and(|n| n.eq_ignore_ascii_case(id) || n.to_lowercase().replace(' ', "_") == id)
    })
}

fn key_from_name(name: &str) -> Option<InputKey> {
    Some(match name {
        "up" => InputKey::Up,
        "down" => InputKey::Down,
        "left" => InputKey::Left,
        "right" => InputKey::Right,
        "ok" | "a" => InputKey::Ok,
        "back" | "b" => InputKey::Back,
        "menu" => InputKey::Menu,
        _ => return None,
    })
}

/// Scripted run: taps every key in order (50 ms apart), renders `frames`
/// frames 100 ms apart, writes the last one as PNG.
fn run_headless(kernel: &mut Kernel, args: &Args) {
    let mut t = 1000u32;
    kernel.step(t);
    for name in &args.keys {
        let Some(key) = key_from_name(name) else {
            eprintln!("fri3d: unknown key '{name}'");
            std::process::exit(2);
        };
        let hold = if name.starts_with("long") { 400 } else { 50 };
        kernel.push_raw_input(key, true, t);
        kernel.step(t);
        t += hold;
        kernel.push_raw_input(key, false, t);
        kernel.step(t);
        t += 50;
    }
    for _ in 0..args.frames {
        t += 100;
        kernel.request_render();
        kernel.step(t);
    }
    drain_log(kernel);
    eprintln!(
        "[perf] last render: {} fuel, app memory {} KB",
        kernel.stats.last_render_fuel,
        kernel.stats.app_memory_bytes / 1024
    );
    if let Some(path) = &args.screenshot {
        if let Err(e) = write_png(path, &kernel.framebuffer()) {
            eprintln!("fri3d: {e}");
            std::process::exit(1);
        }
    }
    if !kernel.last_error().is_empty() {
        eprintln!("kernel error: {}", kernel.last_error());
    }
}

fn run_window(kernel: &mut Kernel, settings_path: &Path, start: Instant) {
    const SCALE: usize = 4;
    let (w, h) = (SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize);
    let mut window = Window::new(
        "Fri3d Badge",
        w,
        h,
        WindowOptions {
            scale: Scale::X4,
            ..WindowOptions::default()
        },
    )
    .expect("open window");
    window.set_target_fps(60);
    let _ = SCALE;

    let bindings: [(&[Key], InputKey); 7] = [
        (&[Key::Up, Key::W], InputKey::Up),
        (&[Key::Down, Key::S], InputKey::Down),
        (&[Key::Left, Key::A], InputKey::Left),
        (&[Key::Right, Key::D], InputKey::Right),
        (&[Key::Z, Key::Enter], InputKey::Ok),
        (&[Key::X, Key::Backspace], InputKey::Back),
        (&[Key::M, Key::Escape], InputKey::Menu),
    ];

    let mut rgb = vec![BG; w * h];
    let mut last_error = String::new();
    let mut shot = 0u32;
    let mut settings_img = [0u8; IMAGE_LEN];
    let mut last_perf = Instant::now();

    while window.is_open() {
        let now = start.elapsed().as_millis() as u32;
        for (keys, input) in &bindings {
            let down = keys.iter().any(|k| window.is_key_down(*k));
            kernel.push_raw_input(*input, down, now);
        }
        if window.is_key_pressed(Key::F12, minifb::KeyRepeat::No) {
            let path = format!("screenshot_{shot}.png");
            shot += 1;
            match write_png(Path::new(&path), &kernel.framebuffer()) {
                Ok(()) => println!("wrote {path}"),
                Err(e) => eprintln!("{e}"),
            }
        }

        let t0 = Instant::now();
        let result = kernel.step(now);
        let dt = t0.elapsed();
        if result.frame {
            let brightness = kernel.setting("system", "brightness").unwrap_or(100).clamp(10, 100);
            let bg = scale_color(BG, brightness);
            let fb = kernel.framebuffer();
            for (dst, &px) in rgb.iter_mut().zip(fb.iter()) {
                *dst = if px != 0 { FG } else { bg };
            }
            drop(fb);
            if dt > Duration::from_millis(20) || last_perf.elapsed() > Duration::from_secs(5) {
                last_perf = Instant::now();
                println!(
                    "[perf] step {:?}  fuel {}  app mem {} KB",
                    dt,
                    kernel.stats.last_render_fuel,
                    kernel.stats.app_memory_bytes / 1024
                );
            }
        }
        window.update_with_buffer(&rgb, w, h).expect("update window");

        drain_log(kernel);
        if kernel.last_error() != last_error {
            last_error = kernel.last_error().to_string();
            if !last_error.is_empty() {
                eprintln!("[kernel] {last_error}");
            }
        }
        if kernel.take_settings_image(&mut settings_img) {
            if let Some(dir) = settings_path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Err(e) = std::fs::write(settings_path, settings_img) {
                eprintln!("settings: {e}");
            }
        }
    }
}

fn drain_log(kernel: &mut Kernel) {
    while let Some(line) = kernel.take_log_line() {
        println!("[app] {line}");
    }
}

fn scale_color(c: u32, pct: u32) -> u32 {
    let ch = |shift: u32| (((c >> shift) & 0xFF) * pct / 100) << shift;
    ch(16) | ch(8) | ch(0)
}

fn settings_path() -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    home.join(".fri3d-badge").join("settings.bin")
}

/// Greyscale PNG, 0 = black pixel, 255 = white — the convention of
/// tests/visual/apps/*/golden/*.png.
fn write_png(path: &Path, fb: &[u8]) -> Result<(), String> {
    debug_assert_eq!(fb.len(), FRAMEBUFFER_LEN);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
        }
    }
    let file = std::fs::File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), SCREEN_WIDTH, SCREEN_HEIGHT);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| format!("png: {e}"))?;
    let grey: Vec<u8> = fb.iter().map(|&p| if p != 0 { 0 } else { 255 }).collect();
    writer.write_image_data(&grey).map_err(|e| format!("png: {e}"))?;
    Ok(())
}
