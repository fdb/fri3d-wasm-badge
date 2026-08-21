//! Fri3d 2026 badge firmware.
//!
//! The kernel does all the work; this file is the IO layer:
//! CH32 expander (buttons, LCD reset, backlight) over I²C1, ST7789V over
//! SPI2, and a 2× upscale of the 160×120 canvas to the full 320×240 panel.

#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
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
use esp_hal::time::{Instant, Rate};
use esp_backtrace as _;
use esp_hal::Blocking;
use esp_println::println;
use fri3d_kernel::types::InputKey;
use fri3d_kernel::{Kernel, FRAMEBUFFER_LEN, SCREEN_HEIGHT, SCREEN_WIDTH};
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::Builder;

esp_bootloader_esp_idf::esp_app_desc!();

// ---- Settings persistence ---------------------------------------------
//
// The `settings` partition holds one 4 KB sector: magic, length, then the
// kernel's settings image. Rewritten only when a setting changes.

const SETTINGS_MAGIC: &[u8; 4] = b"FSET";
const SECTOR: usize = 4096;
const _: () = assert!(4 + 4 + fri3d_kernel::settings::IMAGE_LEN <= SECTOR);

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

fn settings_load(flash: &mut esp_storage::FlashStorage<'static>, kernel: &mut Kernel) {
    use embedded_storage::ReadStorage;
    let Some((offset, _)) = find_partition(flash, "settings") else {
        println!("[fri3d] no settings partition");
        return;
    };
    let mut sector = [0u8; SECTOR];
    if flash.read(offset, &mut sector).is_err() || &sector[..4] != SETTINGS_MAGIC {
        println!("[fri3d] settings: empty");
        return;
    }
    let len = u32::from_le_bytes([sector[4], sector[5], sector[6], sector[7]]) as usize;
    if len > SECTOR - 8 {
        return;
    }
    kernel.load_settings(&sector[8..8 + len]);
    println!("[fri3d] settings: loaded {} B", len);
}

fn settings_store(flash: &mut esp_storage::FlashStorage<'static>, image: &[u8]) {
    use embedded_storage::Storage;
    let Some((offset, _)) = find_partition(flash, "settings") else { return };
    let mut sector = [0xFFu8; SECTOR];
    sector[..4].copy_from_slice(SETTINGS_MAGIC);
    sector[4..8].copy_from_slice(&(image.len() as u32).to_le_bytes());
    sector[8..8 + image.len()].copy_from_slice(image);
    // `Storage::write` erases the sector first.
    println!("[fri3d] settings: store -> {:?}", flash.write(offset, &sector).is_ok());
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
    esp_alloc::heap_allocator!(size: 96 * 1024);
    let mut delay = Delay::new();
    // Give the host a moment to attach to USB-JTAG so early log lines land.
    delay.delay_ms(300);
    println!("[fri3d] boot, internal heap ready");

    // N16R8: 8 MB octal PSRAM.
    let psram_config = esp_hal::psram::PsramConfig {
        mode: esp_hal::psram::PsramMode::Auto,
        ..Default::default()
    };
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram, psram_config);
    println!("[fri3d] psram ready, free heap {} B", esp_alloc::HEAP.free());

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
            settings_store(&mut flash, &settings_img);
        }

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
