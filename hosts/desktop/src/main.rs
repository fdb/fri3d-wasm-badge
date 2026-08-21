//! Desktop host.
//!
//! Interactive:  `cargo run -p fri3d-host-desktop --release`
//! Headless:     `fri3d --headless --app snake --keys ok,down --frames 3 --screenshot out.png`
//!
//! Keys: arrows / WASD = d-pad, Z / Enter = OK, X / Backspace = Back,
//! M / Escape = Menu (home), F12 = screenshot.

use fri3d_kernel::settings::IMAGE_LEN;
use fri3d_kernel::types::InputKey;
use fri3d_kernel::net::{NetRequest, Sim as NetSim};
use fri3d_kernel::wifi::{Sim, IMAGE_LEN as WIFI_IMAGE_LEN};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
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
    real_net: bool,
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
        real_net: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        let mut val = || it.next().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--headless" => a.headless = true,
            "--list" => a.list = true,
            "--real-net" => a.real_net = true,
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

    let settings_path = state_path("settings.bin");
    if let Ok(img) = std::fs::read(&settings_path) {
        kernel.load_settings(&img);
    }
    let wifi_path = state_path("wifi.bin");
    if let Ok(img) = std::fs::read(&wifi_path) {
        kernel.load_wifi(&img);
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
    run_window(&mut kernel, &settings_path, &wifi_path, start);
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
    let mut sim = Sim::new();
    let mut net_sim = NetSim::new();
    // --real-net: real sockets on a thread, so frames wait in wall time.
    let mut net = NetDriver::default();
    let mut t = 1000u32;
    kernel.step(t);
    for name in &args.keys {
        let Some(key) = key_from_name(name) else {
            eprintln!("fri3d: unknown key '{name}'");
            std::process::exit(2);
        };
        let hold = if name.starts_with("long") { 400 } else { 50 };
        sim.service(&mut kernel.wifi_mut(), t);
        net_sim.service(&mut kernel.net_mut(), t);
        kernel.push_raw_input(key, true, t);
        kernel.step(t);
        t += hold;
        kernel.push_raw_input(key, false, t);
        kernel.step(t);
        t += 50;
    }
    for _ in 0..args.frames {
        t += 100;
        sim.service(&mut kernel.wifi_mut(), t);
        if args.real_net {
            std::thread::sleep(Duration::from_millis(100));
            net.service(kernel);
        } else {
            net_sim.service(&mut kernel.net_mut(), t);
        }
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

fn run_window(kernel: &mut Kernel, settings_path: &Path, wifi_path: &Path, start: Instant) {
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
    let mut wifi_img = [0u8; WIFI_IMAGE_LEN];
    let mut sim = Sim::new();
    let mut net = NetDriver::default();
    let mut last_perf = Instant::now();

    while window.is_open() {
        let now = start.elapsed().as_millis() as u32;
        sim.service(&mut kernel.wifi_mut(), now);
        net.service(kernel);
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
            persist(settings_path, &settings_img);
        }
        if kernel.take_wifi_image(&mut wifi_img) {
            persist(wifi_path, &wifi_img);
        }
    }
}

fn persist(path: &Path, bytes: &[u8]) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(path, bytes) {
        eprintln!("{}: {e}", path.display());
    }
}

/// Real network operations on a worker thread: the kernel's `Probe` and
/// `Download` primitives over std sockets (plain HTTP/1.0, body discarded).
#[derive(Default)]
struct NetDriver {
    rx: Option<Receiver<NetMsg>>,
    cancel: Option<Arc<AtomicBool>>,
}

enum NetMsg {
    Progress(u32),
    Done(bool),
}

impl NetDriver {
    fn service(&mut self, kernel: &mut Kernel) {
        if let Some(rx) = &self.rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    NetMsg::Progress(b) => kernel.net_progress(b),
                    NetMsg::Done(ok) => {
                        kernel.net_done(ok);
                        self.rx = None;
                        self.cancel = None;
                        break;
                    }
                }
            }
        }
        let Some(req) = kernel.take_net_request() else { return };
        if let Some(c) = &self.cancel {
            c.store(true, Ordering::Relaxed);
        }
        self.rx = None;
        self.cancel = None;
        let job: Box<dyn FnOnce(&AtomicBool, &dyn Fn(u32)) -> bool + Send> = match req {
            NetRequest::Cancel => return,
            NetRequest::Probe { ip, port } => Box::new(move |_, _| tcp_probe(ip, port)),
            NetRequest::Download { url } => {
                let url = url.to_string();
                Box::new(move |cancel, progress| http_download(&url, cancel, progress))
            }
        };
        let (tx, rx) = channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let c2 = Arc::clone(&cancel);
        std::thread::spawn(move || {
            let tx2 = tx.clone();
            let ok = job(&c2, &move |b| {
                let _ = tx2.send(NetMsg::Progress(b));
            });
            let _ = tx.send(NetMsg::Done(ok));
        });
        self.rx = Some(rx);
        self.cancel = Some(cancel);
    }
}

fn tcp_probe(ip: [u8; 4], port: u16) -> bool {
    let addr = SocketAddr::from((ip, port));
    TcpStream::connect_timeout(&addr, Duration::from_secs(3)).is_ok()
}

/// GET `url` over plain HTTP, count the body, drop it. True on a 2xx
/// response with at least one body byte.
fn http_download(url: &str, cancel: &AtomicBool, progress: &dyn Fn(u32)) -> bool {
    let Some(rest) = url.strip_prefix("http://") else { return false };
    let (hostport, path) = rest.split_once('/').map_or((rest, "/"), |(h, _)| (h, &rest[h.len()..]));
    let host = hostport.split(':').next().unwrap_or(hostport);
    let addr = if hostport.contains(':') { hostport.to_string() } else { format!("{hostport}:80") };
    let Some(addr) = addr.to_socket_addrs().ok().and_then(|mut a| a.next()) else { return false };
    let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_secs(5)) else { return false };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let req = format!("GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    if stream.write_all(req.as_bytes()).is_err() {
        return false;
    }
    let mut buf = vec![0u8; 64 * 1024];
    let mut head = Vec::new();
    let mut in_body = false;
    let mut ok_status = false;
    let mut body = 0u32;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return false;
        }
        let n = match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => return false,
        };
        if in_body {
            body += n as u32;
        } else {
            head.extend_from_slice(&buf[..n]);
            if let Some(end) = head.windows(4).position(|w| w == b"\r\n\r\n") {
                let status = std::str::from_utf8(&head[..end]).unwrap_or("");
                ok_status = status.split_whitespace().nth(1).is_some_and(|c| c.starts_with('2'));
                if !ok_status {
                    eprintln!("speedtest: {}", status.lines().next().unwrap_or("bad response"));
                    return false;
                }
                in_body = true;
                body = (head.len() - end - 4) as u32;
            }
        }
        progress(body);
    }
    ok_status && body > 0
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

fn state_path(name: &str) -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    home.join(".fri3d-badge").join(name)
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
