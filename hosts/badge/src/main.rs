//! Fri3d 2026 badge firmware.
//!
//! The kernel does all the work; this file is the IO layer:
//! CH32 expander (buttons, LCD reset, backlight) over I²C1, ST7789V over
//! SPI2, a 2× upscale of the 160×120 canvas to the full 320×240 panel, and
//! the Wi-Fi radio behind the kernel's three primitives (scan, connect,
//! disconnect).

#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_hal::delay::DelayNs;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::spi::Mode;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::peripherals::WIFI;
use esp_hal::time::{Instant, Rate};
use esp_hal::timer::timg::TimerGroup;
use esp_backtrace as _;
use esp_hal::Blocking;
use esp_println::println;
use embassy_net::dns::DnsQueryType;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv4Address, Stack, StackResources};
use esp_radio::wifi::ap::AccessPointInfo;
use esp_radio::wifi::scan::ScanConfig;
use esp_radio::wifi::sta::StationConfig;
use esp_radio::wifi::{AuthenticationMethod, Config as WifiConfig, WifiController, WifiError};
use fri3d_kernel::types::InputKey;
use fri3d_kernel::net::NetRequest;
use fri3d_kernel::wifi::{ScanEntry, WifiRequest};
use fri3d_kernel::{Kernel, FRAMEBUFFER_LEN, SCREEN_HEIGHT, SCREEN_WIDTH};
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::Builder;

esp_bootloader_esp_idf::esp_app_desc!();

/// After a panic esp-backtrace has printed the message and the frames;
/// a frozen badge helps nobody, so reboot. The delay keeps the log
/// readable on a monitor and avoids a tight reboot loop.
#[no_mangle]
pub extern "Rust" fn custom_halt() -> ! {
    println!("[fri3d] panic: rebooting in 3 s");
    Delay::new().delay_ms(3000);
    esp_hal::system::software_reset()
}

// ---- Persistence ---------------------------------------------------------
//
// The `settings` partition holds one 4 KB sector per blob: magic, length,
// payload. Sector 0 is the kernel settings image, sector 1 the saved
// Wi-Fi networks. A sector is rewritten only when its blob changes.

const SECTOR: usize = 4096;
const SETTINGS_SECTOR: u32 = 0;
const SETTINGS_MAGIC: &[u8; 4] = b"FSET";
const WIFI_SECTOR: u32 = 1;
const WIFI_MAGIC: &[u8; 4] = b"FWIF";
const _: () = assert!(4 + 4 + fri3d_kernel::settings::IMAGE_LEN <= SECTOR);
const _: () = assert!(4 + 4 + fri3d_kernel::wifi::IMAGE_LEN <= SECTOR);

fn find_partition(
    flash: &mut esp_storage::FlashStorage<'static>,
    label: &str,
) -> Option<(u32, u32)> {
    use esp_bootloader_esp_idf::partitions::read_partition_table;
    let mut pt_buf = [0u8; esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN];
    let pt = read_partition_table(flash, &mut pt_buf).ok()?;
    let found = pt
        .iter()
        .find(|p| p.label_as_str() == label)
        .map(|p| (p.offset(), p.len()));
    found
}

/// Read one blob sector into `out`; returns the payload length.
fn blob_load(
    flash: &mut esp_storage::FlashStorage<'static>,
    sector: u32,
    magic: &[u8; 4],
    out: &mut [u8; SECTOR],
) -> Option<usize> {
    use embedded_storage::ReadStorage;
    let (offset, _) = find_partition(flash, "settings")?;
    let offset = offset + sector * SECTOR as u32;
    if flash.read(offset, out).is_err() || &out[..4] != magic {
        return None;
    }
    let len = u32::from_le_bytes([out[4], out[5], out[6], out[7]]) as usize;
    (len <= SECTOR - 8).then_some(len)
}

fn blob_store(
    flash: &mut esp_storage::FlashStorage<'static>,
    sector: u32,
    magic: &[u8; 4],
    payload: &[u8],
) {
    use embedded_storage::Storage;
    let Some((offset, _)) = find_partition(flash, "settings") else { return };
    let offset = offset + sector * SECTOR as u32;
    let mut buf = [0xFFu8; SECTOR];
    buf[..4].copy_from_slice(magic);
    buf[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    buf[8..8 + payload.len()].copy_from_slice(payload);
    // `Storage::write` erases the sector first.
    println!("[fri3d] {} store -> {:?}", core::str::from_utf8(magic).unwrap_or("?"), flash.write(offset, &buf).is_ok());
}

fn settings_load(flash: &mut esp_storage::FlashStorage<'static>, kernel: &mut Kernel) {
    let mut sector = [0u8; SECTOR];
    match blob_load(flash, SETTINGS_SECTOR, SETTINGS_MAGIC, &mut sector) {
        Some(len) => {
            kernel.load_settings(&sector[8..8 + len]);
            println!("[fri3d] settings: loaded {} B", len);
        }
        None => println!("[fri3d] settings: empty"),
    }
    match blob_load(flash, WIFI_SECTOR, WIFI_MAGIC, &mut sector) {
        Some(len) => {
            kernel.load_wifi(&sector[8..8 + len]);
            println!("[fri3d] wifi: loaded {} B", len);
        }
        None => println!("[fri3d] wifi: no saved networks"),
    }
}

/// Report which OTA slot booted and, once the kernel is up, mark the image
/// valid so the bootloader's rollback never reverts a working firmware.
fn ota_confirm_boot(flash: &mut esp_storage::FlashStorage<'static>) {
    use esp_bootloader_esp_idf::ota::OtaImageState;
    use esp_bootloader_esp_idf::ota_updater::OtaUpdater;
    let mut pt_buf = [0u8; esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN];
    match OtaUpdater::new(flash, &mut pt_buf) {
        Ok(mut ota) => {
            let slot = ota.selected_partition();
            let state = ota.current_ota_state();
            println!("[fri3d] ota slot {:?} state {:?}", slot, state);
            if matches!(state, Ok(OtaImageState::New) | Ok(OtaImageState::PendingVerify)) {
                println!("[fri3d] ota mark valid -> {:?}", ota.set_current_ota_state(OtaImageState::Valid));
            }
        }
        Err(e) => println!("[fri3d] no OTA layout ({:?}); single-app partition table", e),
    }
}

// ---- Panel geometry -------------------------------------------------------

const LCD_W: u16 = 320;
const LCD_H: u16 = 240;
const SCALE: u16 = 2;
const FB_W: u16 = SCREEN_WIDTH as u16;
const FB_H: u16 = SCREEN_HEIGHT as u16;
const UP_W: u16 = FB_W * SCALE; // 256
const UP_H: u16 = FB_H * SCALE; // 128
const CANVAS_X: u16 = (LCD_W - UP_W) / 2; // 0
const CANVAS_Y: u16 = (LCD_H - UP_H) / 2; // 0

// Flipper-style amber backlight, black pixels.
const AMBER: Rgb565 = Rgb565::new(0xFF >> 3, 0x82 >> 2, 0x00 >> 3);
const INK: Rgb565 = Rgb565::BLACK;

// ---- CH32 expander --------------------------------------------------------

const EXPANDER_ADDR: u8 = 0x50;
const REG_INPUTS: u8 = 0x04;
const REG_LCD_BRIGHTNESS: u8 = 0x12;
const REG_CONFIG: u8 = 0x16;

/// Bit positions in the inputs register (1 = pressed).
const BIT_JOY_RIGHT: u16 = 1 << 10;
const BIT_JOY_LEFT: u16 = 1 << 9;
const BIT_JOY_DOWN: u16 = 1 << 8;
const BIT_JOY_UP: u16 = 1 << 7;
const BIT_MENU: u16 = 1 << 6;
const BIT_A: u16 = 1 << 4;
const BIT_X: u16 = 1 << 2;

const KEYMAP: [(u16, InputKey); 7] = [
    (BIT_JOY_UP, InputKey::Up),
    (BIT_JOY_DOWN, InputKey::Down),
    (BIT_JOY_LEFT, InputKey::Left),
    (BIT_JOY_RIGHT, InputKey::Right),
    (BIT_A, InputKey::Ok),
    (BIT_X, InputKey::Back),
    (BIT_MENU, InputKey::Menu),
];

struct Expander {
    i2c: I2c<'static, Blocking>,
}

impl Expander {
    fn read_inputs(&mut self) -> Option<u16> {
        let mut buf = [0u8; 2];
        self.i2c
            .write_read(EXPANDER_ADDR, &[REG_INPUTS], &mut buf)
            .ok()?;
        Some(u16::from_le_bytes(buf))
    }


    fn set_brightness(&mut self, pct: u16) {
        let v = pct.min(100).to_le_bytes();
        let _ = self.i2c.write(EXPANDER_ADDR, &[REG_LCD_BRIGHTNESS, v[0], v[1]]);
    }

    fn set_config(&mut self, bits: u8) -> Result<(), esp_hal::i2c::master::Error> {
        self.i2c.write(EXPANDER_ADDR, &[REG_CONFIG, bits])
    }

    fn read_config(&mut self) -> Option<u8> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(EXPANDER_ADDR, &[REG_CONFIG], &mut buf)
            .ok()?;
        Some(buf[0])
    }

    /// Write a config byte until the CH32 acknowledges and reads it back.
    /// On a cold boot the coprocessor needs up to ~1 s before it answers;
    /// the LCD stays in reset until this lands. Bounded: 100 × 30 ms.
    fn set_config_verified(&mut self, bits: u8, delay: &mut Delay) -> bool {
        for _ in 0..100 {
            if self.set_config(bits).is_ok() && self.read_config() == Some(bits) {
                return true;
            }
            delay.delay_ms(30);
        }
        false
    }
}

// ---- Entry ----------------------------------------------------------------

fn now_ms() -> u32 {
    Instant::now().duration_since_epoch().as_millis() as u32
}

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    // Internal heap for small, hot allocations; PSRAM for wasmi's linear
    // memories and the kernel itself.
    let mut delay = Delay::new();
    // Give the host a moment to attach to USB-JTAG so early log lines land.
    delay.delay_ms(300);
    println!("[fri3d] boot");

    // esp-alloc serves the first region that fits, in registration order.
    // PSRAM first: the kernel and wasmi live there. The internal region
    // only answers requests that demand internal RAM — the Wi-Fi driver's
    // task stacks and DMA buffers — so it cannot be eaten up before the
    // radio starts. N16R8: 8 MB octal PSRAM.
    let psram_config = esp_hal::psram::PsramConfig {
        mode: esp_hal::psram::PsramMode::Auto,
        ..Default::default()
    };
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram, psram_config);
    esp_alloc::heap_allocator!(size: 160 * 1024);
    println!("[fri3d] heaps ready, free {} B", esp_alloc::HEAP.free());

    // The radio driver runs its own tasks; esp-rtos turns this context
    // into the main task and preempts it for them.
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_ints = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_ints.software_interrupt0);

    // -- expander: reset LCD, power aux rail, as MicroPythonOS does --------
    let i2c = I2c::new(
        peripherals.I2C1,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .expect("i2c1 config")
    .with_sda(peripherals.GPIO39)
    .with_scl(peripherals.GPIO42);
    let mut expander = Expander { i2c };
    // aux 3V3 on, LCD + LoRa held in reset; then release the resets.
    let t0 = now_ms();
    let ok1 = expander.set_config_verified(0x01, &mut delay);
    delay.delay_ms(100);
    let ok2 = expander.set_config_verified(0x13, &mut delay);
    println!(
        "[fri3d] expander config 0x01 {} 0x13 {} after {} ms",
        ok1,
        ok2,
        now_ms().wrapping_sub(t0)
    );
    delay.delay_ms(120);
    expander.set_brightness(100);
    println!("[fri3d] expander inputs {:?}", expander.read_inputs());

    // -- display -----------------------------------------------------------
    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(40))
            .with_mode(Mode::_0),
    )
    .expect("spi2 config")
    .with_sck(peripherals.GPIO7)
    .with_mosi(peripherals.GPIO6)
    .with_miso(peripherals.GPIO8);
    let cs = Output::new(peripherals.GPIO5, Level::High, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());
    let spi_dev = ExclusiveDevice::new_no_delay(spi, cs).expect("spi device");

    // mipidsi batches pixel writes through this buffer; one scaled row pair
    // (2 × 256 px × 2 bytes) per flush keeps the SPI bus streaming.
    static mut DI_BUF: [u8; 2048] = [0; 2048];
    #[allow(static_mut_refs)]
    let di_buf = unsafe { &mut DI_BUF };
    let di = SpiInterface::new(spi_dev, dc, di_buf);

    println!("[fri3d] display init...");
    let mut display = match Builder::new(ST7789, di)
        .display_size(240, 320)
        .orientation(Orientation::new().rotate(Rotation::Deg270))
        .color_order(ColorOrder::Rgb)
        .invert_colors(ColorInversion::Inverted)
        .init(&mut delay)
    {
        Ok(d) => d,
        Err(e) => {
            loop {
                println!("[fri3d] display init failed: {:?}", e);
                delay.delay_ms(2000);
            }
        }
    };
    println!("[fri3d] display ready");

    let _ = display.clear(AMBER);

    // -- kernel ------------------------------------------------------------
    let mut kernel = Box::new(Kernel::new());
    if let Err(e) = kernel.set_launcher(fri3d_apps::LAUNCHER) {
        println!("[fri3d] launcher bundle rejected: {:?}", e);
    }
    for app in fri3d_apps::APPS {
        if let Err(e) = kernel.add_app(app) {
            println!("[fri3d] app bundle rejected: {:?}", e);
        }
    }
    let mut flash = esp_storage::FlashStorage::new(peripherals.FLASH);
    settings_load(&mut flash, &mut kernel);
    kernel.boot(now_ms());
    ota_confirm_boot(&mut flash);
    println!(
        "[fri3d] kernel up, {} apps, free heap {} B",
        kernel.app_count(),
        esp_alloc::HEAP.free()
    );

    // -- main loop ---------------------------------------------------------
    static mut LAST_FB: [u8; FRAMEBUFFER_LEN] = [0xFF; FRAMEBUFFER_LEN];
    #[allow(static_mut_refs)]
    let last_fb = unsafe { &mut LAST_FB };

    let mut settings_img = [0u8; fri3d_kernel::settings::IMAGE_LEN];
    let mut wifi_img = [0u8; fri3d_kernel::wifi::IMAGE_LEN];
    let mut radio = Radio::new(peripherals.WIFI);
    let mut net = NetDriver::new();
    let mut last_error_len = 0usize;
    let mut brightness = 100u32;
    let mut last_poll = now_ms();
    let mut last_perf_report = now_ms();

    loop {
        let now = now_ms();

        if now.wrapping_sub(last_poll) >= 5 {
            last_poll = now;
            if let Some(bits) = expander.read_inputs() {
                for (bit, key) in KEYMAP {
                    kernel.push_raw_input(key, bits & bit != 0, now);
                }
            }
        }

        let t0 = now_ms();
        let step = kernel.step(now);
        let dt = now_ms().wrapping_sub(t0);

        if step.frame {
            let changed = {
                let fb = kernel.framebuffer();
                let changed = fb[..] != last_fb[..];
                if changed {
                    last_fb.copy_from_slice(&fb);
                }
                changed
            };
            if changed {
                blit(&mut display, last_fb);
            }
            if dt > 30 || now.wrapping_sub(last_perf_report) > 5000 {
                last_perf_report = now;
                println!(
                    "[perf] step {} ms  fuel {}  app mem {} KB  free heap {} B",
                    dt,
                    kernel.stats.last_render_fuel,
                    kernel.stats.app_memory_bytes / 1024,
                    esp_alloc::HEAP.free()
                );
            }
        }

        while let Some(line) = kernel.take_log_line() {
            println!("[app] {}", line.as_str());
        }
        let err = kernel.last_error();
        if err.len() != last_error_len {
            last_error_len = err.len();
            if !err.is_empty() {
                println!("[kernel] {}", err);
            }
        }

        if kernel.take_settings_image(&mut settings_img) {
            blob_store(&mut flash, SETTINGS_SECTOR, SETTINGS_MAGIC, &settings_img);
        }
        if kernel.take_wifi_image(&mut wifi_img) {
            blob_store(&mut flash, WIFI_SECTOR, WIFI_MAGIC, &wifi_img);
        }
        radio.poll(&mut kernel);
        net.poll(&mut kernel, radio.is_up());

        let wanted = kernel.setting("system", "brightness").unwrap_or(100).clamp(5, 100);
        if wanted != brightness {
            brightness = wanted;
            expander.set_brightness(brightness as u16);
        }

        if now.wrapping_sub(last_perf_report) > 5000 {
            last_perf_report = now;
            println!("[fri3d] alive t={} ms, inputs {:?}", now, expander.read_inputs());
        }

        // Idle: nothing to draw and no input. A short sleep keeps the poll
        // cadence without spinning the core flat out.
        if !step.frame {
            delay.delay_ms(2);
        }
    }
}

// ---- Wi-Fi radio ----------------------------------------------------------
//
// esp-radio's scan/connect are async. The main loop is not, so each request
// becomes one boxed future that is polled once per iteration with a no-op
// waker; the driver's own tasks make progress under esp-rtos in between.
// The controller is created on the first request, so a badge with Wi-Fi
// off never powers the radio.

type Ctrl = &'static mut WifiController<'static>;

enum Outcome {
    Scan(Result<Vec<AccessPointInfo>, WifiError>),
    Connect(bool),
    Disconnect,
}

struct Radio {
    peripheral: Option<WIFI<'static>>,
    ctrl: Option<Ctrl>,
    inflight: Option<Pin<Box<dyn Future<Output = (Ctrl, Outcome)>>>>,
    was_connected: bool,
}

impl Radio {
    fn new(peripheral: WIFI<'static>) -> Self {
        Self { peripheral: Some(peripheral), ctrl: None, inflight: None, was_connected: false }
    }

    fn controller(&mut self) -> Option<Ctrl> {
        if let Some(c) = self.ctrl.take() {
            return Some(c);
        }
        let periph = self.peripheral.take()?;
        match WifiController::new(periph, Default::default()) {
            Ok(c) => {
                println!("[wifi] radio up, free heap {} B", esp_alloc::HEAP.free());
                Some(Box::leak(Box::new(c)))
            }
            Err(e) => {
                println!("[wifi] init failed: {:?}", e);
                None
            }
        }
    }

    /// The radio exists and the station is associated.
    fn is_up(&self) -> bool {
        self.was_connected
    }

    fn poll(&mut self, kernel: &mut Kernel) {
        if let Some(f) = self.inflight.as_mut() {
            let mut cx = Context::from_waker(Waker::noop());
            let Poll::Ready((ctrl, outcome)) = f.as_mut().poll(&mut cx) else { return };
            self.inflight = None;
            self.ctrl = Some(ctrl);
            match outcome {
                Outcome::Scan(Ok(aps)) => {
                    let mut entries: heapless::Vec<ScanEntry, { fri3d_kernel::limits::WIFI_SCAN_MAX }> =
                        heapless::Vec::new();
                    for ap in &aps {
                        let mut ssid = fri3d_kernel::wifi::Ssid::new();
                        if ap.ssid.as_str().is_empty() || ssid.push_str(ap.ssid.as_str()).is_err() {
                            continue;
                        }
                        if entries.iter().any(|e| e.ssid == ssid) {
                            continue;
                        }
                        let secure = !matches!(ap.auth_method, Some(AuthenticationMethod::None));
                        let _ = entries.push(ScanEntry { ssid, rssi: ap.signal_strength, secure });
                    }
                    println!("[wifi] scan: {} networks", entries.len());
                    kernel.wifi_scan_done(&entries);
                }
                Outcome::Scan(Err(e)) => {
                    println!("[wifi] scan failed: {:?}", e);
                    kernel.wifi_scan_done(&[]);
                }
                Outcome::Connect(ok) => {
                    println!("[wifi] connect -> {}", ok);
                    self.was_connected = ok;
                    kernel.wifi_connect_done(ok);
                }
                Outcome::Disconnect => self.was_connected = false,
            }
        }

        if let Some(c) = &self.ctrl {
            let connected = c.is_connected();
            if self.was_connected && !connected {
                println!("[wifi] link lost");
                kernel.wifi_link_lost();
            }
            self.was_connected = connected;
        }

        let Some(req) = kernel.take_wifi_request() else { return };
        match req {
            WifiRequest::Scan => {
                let Some(ctrl) = self.controller() else {
                    kernel.wifi_scan_done(&[]);
                    return;
                };
                self.inflight = Some(Box::pin(async move {
                    let r = ctrl.scan_async(&ScanConfig::default().with_max(fri3d_kernel::limits::WIFI_SCAN_MAX)).await;
                    (ctrl, Outcome::Scan(r))
                }));
            }
            WifiRequest::Connect { ssid, password } => {
                let Some(ctrl) = self.controller() else {
                    kernel.wifi_connect_done(false);
                    return;
                };
                println!("[wifi] connecting to '{}'", ssid.as_str());
                let auth = if password.is_empty() { AuthenticationMethod::None } else { AuthenticationMethod::Wpa2Personal };
                let cfg = StationConfig::default()
                    .with_ssid(ssid.as_str())
                    .with_password(password.as_str().into())
                    .with_auth_method(auth);
                if let Err(e) = ctrl.set_config(&WifiConfig::Station(cfg)) {
                    println!("[wifi] config rejected: {:?}", e);
                    self.ctrl = Some(ctrl);
                    kernel.wifi_connect_done(false);
                    return;
                }
                self.inflight = Some(Box::pin(async move {
                    let r = ctrl.connect_async().await;
                    if let Err(e) = &r {
                        println!("[wifi] connect error: {:?}", e);
                    }
                    (ctrl, Outcome::Connect(r.is_ok()))
                }));
            }
            WifiRequest::Disconnect => {
                // Radio never started: nothing to drop.
                let Some(ctrl) = self.ctrl.take() else { return };
                self.inflight = Some(Box::pin(async move {
                    let _ = ctrl.disconnect_async().await;
                    (ctrl, Outcome::Disconnect)
                }));
            }
        }
    }
}

// ---- IP stack -------------------------------------------------------------
//
// embassy-net over the esp-radio station interface, DHCP. The stack's
// runner and the one operation in flight are boxed futures polled from the
// main loop, like the radio. The stack is built on the first request after
// the station associated; it is never torn down.

struct NetDriver {
    stack: Option<Stack<'static>>,
    runner: Option<Pin<Box<dyn Future<Output = ()>>>>,
    op: Option<Pin<Box<dyn Future<Output = bool>>>>,
    progress: Arc<AtomicU32>,
}

const NET_TIMEOUT: embassy_time::Duration = embassy_time::Duration::from_secs(10);

impl NetDriver {
    fn new() -> Self {
        Self { stack: None, runner: None, op: None, progress: Arc::new(AtomicU32::new(0)) }
    }

    fn stack(&mut self) -> Stack<'static> {
        if let Some(s) = self.stack {
            return s;
        }
        let rng = esp_hal::rng::Rng::new();
        let seed = (rng.random() as u64) << 32 | rng.random() as u64;
        let resources: &'static mut StackResources<4> = Box::leak(Box::new(StackResources::new()));
        let (stack, mut runner) = embassy_net::new(
            esp_radio::wifi::Interface::station(),
            embassy_net::Config::dhcpv4(Default::default()),
            resources,
            seed,
        );
        self.runner = Some(Box::pin(async move { runner.run().await }));
        self.stack = Some(stack);
        println!("[net] stack up");
        stack
    }

    fn poll(&mut self, kernel: &mut Kernel, link_up: bool) {
        let mut cx = Context::from_waker(Waker::noop());
        if let Some(r) = self.runner.as_mut() {
            let _ = r.as_mut().poll(&mut cx);
        }
        if let Some(op) = self.op.as_mut() {
            kernel.net_progress(self.progress.load(Ordering::Relaxed));
            if let Poll::Ready(ok) = op.as_mut().poll(&mut cx) {
                self.op = None;
                println!("[net] done -> {}", ok);
                kernel.net_progress(self.progress.load(Ordering::Relaxed));
                kernel.net_done(ok);
            }
        }
        let Some(req) = kernel.take_net_request() else { return };
        self.op = None;
        self.progress.store(0, Ordering::Relaxed);
        if matches!(req, NetRequest::Cancel) {
            return;
        }
        if !link_up {
            println!("[net] request without a link");
            kernel.net_done(false);
            return;
        }
        let stack = self.stack();
        let progress = Arc::clone(&self.progress);
        self.op = Some(match req {
            NetRequest::Probe { ip, port } => Box::pin(tcp_probe(stack, ip, port)),
            NetRequest::Download { url } => Box::pin(http_download(stack, url, progress)),
            NetRequest::Cancel => unreachable!(),
        });
    }
}

async fn tcp_probe(stack: Stack<'static>, ip: [u8; 4], port: u16) -> bool {
    stack.wait_config_up().await;
    let mut rx = [0u8; 1024];
    let mut tx = [0u8; 1024];
    let mut socket = TcpSocket::new(stack, &mut rx, &mut tx);
    socket.set_timeout(Some(NET_TIMEOUT));
    let r = socket.connect((Ipv4Address::new(ip[0], ip[1], ip[2], ip[3]), port)).await;
    if let Err(e) = &r {
        println!("[net] probe {:?}:{} -> {:?}", ip, port, e);
    }
    socket.close();
    r.is_ok()
}

/// Plain HTTP/1.0 GET; counts the body into `progress`, keeps nothing.
async fn http_download(stack: Stack<'static>, url: fri3d_kernel::net::Url, progress: Arc<AtomicU32>) -> bool {
    use embedded_io_async::Write;
    let Some(rest) = url.strip_prefix("http://") else {
        println!("[net] only http:// is supported");
        return false;
    };
    let (hostport, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = match hostport.split_once(':') {
        Some((h, p)) => (h, p.parse().unwrap_or(80)),
        None => (hostport, 80u16),
    };
    stack.wait_config_up().await;
    let addr = match stack.dns_query(host, DnsQueryType::A).await {
        Ok(addrs) if !addrs.is_empty() => addrs[0],
        other => {
            println!("[net] dns {} -> {:?}", host, other);
            return false;
        }
    };
    println!("[net] {} -> {}", host, addr);
    let mut rx = [0u8; 16 * 1024];
    let mut tx = [0u8; 1024];
    let mut socket = TcpSocket::new(stack, &mut rx, &mut tx);
    socket.set_timeout(Some(NET_TIMEOUT));
    if let Err(e) = socket.connect((addr, port)).await {
        println!("[net] connect -> {:?}", e);
        return false;
    }
    let mut req: heapless::String<256> = heapless::String::new();
    let _ = core::fmt::write(&mut req, format_args!("GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n"));
    if socket.write_all(req.as_bytes()).await.is_err() {
        return false;
    }
    let mut buf = [0u8; 4096];
    let mut head: heapless::Vec<u8, 2048> = heapless::Vec::new();
    let mut in_body = false;
    let mut body = 0u32;
    loop {
        let n = match socket.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                println!("[net] read -> {:?}", e);
                return false;
            }
        };
        if in_body {
            body += n as u32;
        } else {
            let _ = head.extend_from_slice(&buf[..n.min(head.capacity() - head.len())]);
            if let Some(end) = head.windows(4).position(|w| w == b"\r\n\r\n") {
                let status = core::str::from_utf8(&head[..end]).unwrap_or("");
                let ok = status.split_whitespace().nth(1).is_some_and(|c| c.starts_with('2'));
                println!("[net] {}", status.lines().next().unwrap_or("?"));
                if !ok {
                    return false;
                }
                in_body = true;
                body = (head.len() - end - 4) as u32;
            } else if head.is_full() {
                return false;
            }
        }
        progress.store(body, Ordering::Relaxed);
    }
    socket.close();
    in_body && body > 0
}

// ---- Drawing --------------------------------------------------------------

type Disp = mipidsi::Display<
    SpiInterface<
        'static,
        ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, embedded_hal_bus::spi::NoDelay>,
        Output<'static>,
    >,
    ST7789,
    mipidsi::NoResetPin,
>;

/// 2× nearest-neighbour upscale, streamed one canvas row (= two panel rows)
/// at a time so no full-size frame buffer is needed.
fn blit(display: &mut Disp, fb: &[u8]) {
    let ink: u16 = RawU16::from(INK).into_inner();
    let amber: u16 = RawU16::from(AMBER).into_inner();
    for y in 0..FB_H {
        let row = &fb[(y as usize) * FB_W as usize..][..FB_W as usize];
        let line = row.iter().flat_map(move |&px| {
            let c = if px != 0 { ink } else { amber };
            [c, c]
        });
        // Two identical panel rows per canvas row.
        let pixels = line.clone().chain(line).map(|raw| Rgb565::from(RawU16::new(raw)));
        let sy = CANVAS_Y + y * SCALE;
        if display
            .set_pixels(CANVAS_X, sy, CANVAS_X + UP_W - 1, sy + SCALE - 1, pixels)
            .is_err()
        {
            println!("[fri3d] blit error at row {}", y);
            return;
        }
    }
}
