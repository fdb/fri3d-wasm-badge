//! Settings — Wi-Fi, brightness, sound, about. A system app: it may write
//! the `system` settings namespace and drive the radio.
//!
//! Wi-Fi flow: the list merges the last scan (strongest first) with saved
//! networks that are out of range. Pick a network: a saved one offers
//! connect / change password / forget; an unknown secured one asks for a
//! password on the on-screen keyboard and connects; an open one connects
//! at once. "Add network" types an SSID and password for hidden networks.
//! Long-press Back cancels a keyboard. The kernel re-renders this app
//! when a scan finishes or the link state changes, so there is no timer.
#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;
use fri3d_wasm_api::imgui::UiVirtualKeyboard;
use fri3d_wasm_api::{align, color, font, imgui, input, wifi};

const NS: &str = "system";
const KEY_BRIGHTNESS: &str = "brightness";
const KEY_SOUND: &str = "sound";

// Main menu.
const ITEM_WIFI: i16 = 0;
const ITEM_BRIGHTNESS: i16 = 1;
const ITEM_SOUND: i16 = 2;
const ITEM_ABOUT: i16 = 3;
const ITEM_COUNT: i16 = 4;

// Wi-Fi screen: fixed rows, then one row per network.
const WIFI_TOGGLE: i16 = 0;
const WIFI_SCAN: i16 = 1;
const WIFI_ADD: i16 = 2;
const WIFI_FIRST_NET: i16 = 3;

// Network screen.
const NET_CONNECT: i16 = 0;
const NET_PASSWORD: i16 = 1;
const NET_FORGET: i16 = 2;

const MENU_ROWS: i16 = 7;
const LABEL_CHARS: usize = 16;

#[derive(Copy, Clone, PartialEq, Eq)]
enum Screen {
    Menu,
    Wifi,
    Network,
    Keyboard,
    About,
}

/// What the keyboard text is for, and where to go afterwards.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Typing {
    /// Password for `selected`; save and connect.
    Password,
    /// SSID of a hidden network; then ask its password.
    NewSsid,
}

#[derive(Copy, Clone)]
struct State {
    screen: Screen,
    menu_scroll: i16,
    wifi_scroll: i16,
    net_scroll: i16,
    brightness: u32,
    sound: bool,
    selected: wifi::Ssid,
    typing: Typing,
}

static STATE: api::AppCell<State> = api::AppCell::new(State {
    screen: Screen::Menu,
    menu_scroll: 0,
    wifi_scroll: 0,
    net_scroll: 0,
    brightness: 100,
    sound: true,
    selected: wifi::Ssid::empty(),
    typing: Typing::Password,
});

static KEYBOARD: api::AppCell<UiVirtualKeyboard<{ wifi::PASSWORD_LEN + 1 }>> =
    api::AppCell::new(UiVirtualKeyboard::new());

fn on_start_impl() {
    let mut s = STATE.get();
    s.brightness = api::settings_get_u32(NS, KEY_BRIGHTNESS, 100).clamp(10, 100);
    s.sound = api::settings_get_u32(NS, KEY_SOUND, 1) != 0;
    STATE.set(s);
}

fn render_impl() {
    let mut s = STATE.get();
    imgui::ui_begin();
    match s.screen {
        Screen::Menu => render_menu(&mut s),
        Screen::Wifi => render_wifi(&mut s),
        Screen::Network => render_network(&mut s),
        Screen::Keyboard => render_keyboard(&mut s),
        Screen::About => render_about(),
    }
    imgui::ui_end();
    STATE.set(s);
}

// -- main menu ---------------------------------------------------------------

fn render_menu(s: &mut State) {
    imgui::ui_label("Settings", font::PRIMARY, align::CENTER);
    imgui::ui_separator();

    let wifi_value = match wifi::status() {
        wifi::STATUS_OFF => "Off",
        wifi::STATUS_CONNECTED => "Connected",
        _ => "On",
    };
    let mut pct = Small::new();
    pct.push(s.brightness);
    pct.push_str("%");

    imgui::ui_menu_begin(&mut s.menu_scroll, MENU_ROWS, ITEM_COUNT);
    if imgui::ui_menu_item_value("Wi-Fi", wifi_value, ITEM_WIFI) {
        s.screen = Screen::Wifi;
        imgui::ui_set_focus(0);
    }
    imgui::ui_menu_item_value("Brightness", pct.as_str(), ITEM_BRIGHTNESS);
    imgui::ui_menu_item_value("Sound", if s.sound { "On" } else { "Off" }, ITEM_SOUND);
    if imgui::ui_menu_item("About", ITEM_ABOUT) {
        s.screen = Screen::About;
    }
    imgui::ui_menu_end();
}

// -- Wi-Fi list ----------------------------------------------------------------

/// Row `i` of the network list: scan results first, then saved networks
/// that the scan did not see.
fn list_row(i: u32) -> Option<(wifi::Ssid, bool)> {
    let scanned = wifi::scan_count();
    if i < scanned {
        return Some((wifi::scan_ssid(i), wifi::scan_secure(i)));
    }
    let mut n = i - scanned;
    for k in 0..wifi::saved_count() {
        let ssid = wifi::saved_ssid(k);
        if (0..scanned).any(|j| wifi::scan_ssid(j) == ssid) {
            continue;
        }
        if n == 0 {
            return Some((ssid, true));
        }
        n -= 1;
    }
    None
}

fn list_len() -> u32 {
    let mut n = wifi::scan_count();
    for k in 0..wifi::saved_count() {
        let ssid = wifi::saved_ssid(k);
        if !(0..wifi::scan_count()).any(|j| wifi::scan_ssid(j) == ssid) {
            n += 1;
        }
    }
    n
}

fn row_status(ssid: &wifi::Ssid, secure: bool) -> &'static str {
    if *ssid == wifi::current_ssid() {
        match wifi::status() {
            wifi::STATUS_CONNECTING => return "connecting",
            wifi::STATUS_CONNECTED => return "connected",
            wifi::STATUS_FAILED => return "failed",
            _ => {}
        }
    }
    if wifi::is_saved(ssid.as_str()) {
        "saved"
    } else if !secure {
        "open"
    } else {
        ""
    }
}

fn render_wifi(s: &mut State) {
    imgui::ui_label("Wi-Fi", font::PRIMARY, align::CENTER);
    imgui::ui_separator();

    let on = wifi::enabled();
    let nets = if on { list_len() } else { 0 };
    let total = if on { WIFI_FIRST_NET + nets as i16 } else { 1 };

    imgui::ui_menu_begin(&mut s.wifi_scroll, MENU_ROWS, total);
    imgui::ui_menu_item_value("Wi-Fi", if on { "On" } else { "Off" }, WIFI_TOGGLE);
    if on {
        let scan_value = if wifi::scanning() { "scanning..." } else { "" };
        if imgui::ui_menu_item_value("Scan", scan_value, WIFI_SCAN) {
            wifi::scan();
        }
        if imgui::ui_menu_item("Add network", WIFI_ADD) {
            s.typing = Typing::NewSsid;
            open_keyboard(s, "", 1);
        }
        for i in 0..nets {
            let Some((ssid, secure)) = list_row(i) else { break };
            let label = clip(ssid.as_str(), LABEL_CHARS);
            let status = row_status(&ssid, secure);
            if imgui::ui_menu_item_value(label, status, WIFI_FIRST_NET + i as i16) {
                select_network(s, ssid, secure);
            }
        }
    }
    imgui::ui_menu_end();
}

/// OK on a network row.
fn select_network(s: &mut State, ssid: wifi::Ssid, secure: bool) {
    s.selected = ssid;
    if wifi::is_saved(ssid.as_str()) {
        s.screen = Screen::Network;
        s.net_scroll = 0;
        imgui::ui_set_focus(0);
    } else if secure {
        s.typing = Typing::Password;
        open_keyboard(s, "", 8);
    } else {
        wifi::save(ssid.as_str(), "");
        wifi::connect(ssid.as_str());
    }
}

// -- one network -----------------------------------------------------------------

fn render_network(s: &mut State) {
    imgui::ui_label(clip(s.selected.as_str(), 24), font::PRIMARY, align::CENTER);
    imgui::ui_separator();

    let connected = wifi::status() == wifi::STATUS_CONNECTED && wifi::current_ssid() == s.selected;

    imgui::ui_menu_begin(&mut s.net_scroll, MENU_ROWS, 3);
    if imgui::ui_menu_item(if connected { "Disconnect" } else { "Connect" }, NET_CONNECT) {
        if connected {
            wifi::disconnect();
        } else {
            wifi::connect(s.selected.as_str());
        }
        s.screen = Screen::Wifi;
    }
    if imgui::ui_menu_item("Change password", NET_PASSWORD) {
        s.typing = Typing::Password;
        open_keyboard(s, "", 8);
    }
    if imgui::ui_menu_item("Forget", NET_FORGET) {
        wifi::forget(s.selected.as_str());
        s.screen = Screen::Wifi;
        imgui::ui_set_focus(0);
    }
    imgui::ui_menu_end();
}

// -- keyboard ----------------------------------------------------------------------

fn open_keyboard(s: &mut State, initial: &str, min_len: usize) {
    let mut kb = KEYBOARD.get();
    imgui::ui_virtual_keyboard_init(&mut kb, initial);
    imgui::ui_virtual_keyboard_set_min_length(&mut kb, min_len);
    KEYBOARD.set(kb);
    s.screen = Screen::Keyboard;
}

fn render_keyboard(s: &mut State) {
    let mut header = Small::new();
    match s.typing {
        Typing::Password => {
            header.push_str("Password: ");
            header.push_str(clip(s.selected.as_str(), 16));
        }
        Typing::NewSsid => header.push_str("Network name"),
    }
    let mut kb = KEYBOARD.get();
    let submitted = imgui::ui_virtual_keyboard(&mut kb, header.as_str(), api::get_time_ms());
    KEYBOARD.set(kb);
    if !submitted {
        return;
    }
    let kb = KEYBOARD.get();
    match s.typing {
        Typing::Password => {
            wifi::save(s.selected.as_str(), kb.text());
            wifi::connect(s.selected.as_str());
            s.screen = Screen::Wifi;
            imgui::ui_set_focus(0);
        }
        Typing::NewSsid => {
            s.selected = wifi::Ssid::from_str(kb.text());
            s.typing = Typing::Password;
            open_keyboard(s, "", 8);
        }
    }
    api::request_render();
}

// -- about -----------------------------------------------------------------------------

fn render_about() {
    imgui::ui_label("About", font::PRIMARY, align::CENTER);
    imgui::ui_separator();
    api::canvas_set_color(color::BLACK);
    api::canvas_set_font(font::SECONDARY);
    let mut line = Small::new();
    line.push_str("Fri3d WASM badge");
    api::canvas_draw_str(2, 26, line.as_str());
    let mut line = Small::new();
    line.push_str("kernel ABI v");
    line.push(api::kernel_version());
    api::canvas_draw_str(2, 36, line.as_str());
    let mut line = Small::new();
    line.push(api::app_count());
    line.push_str(" apps installed");
    api::canvas_draw_str(2, 46, line.as_str());
    imgui::ui_footer_left("Back");
}

// -- input ---------------------------------------------------------------------------

fn on_input_impl(key: u32, kind: u32) {
    let mut s = STATE.get();
    let short = kind == input::TYPE_SHORT_PRESS;
    let long = kind == input::TYPE_LONG_PRESS;
    let step = kind == input::TYPE_SHORT_PRESS || kind == input::TYPE_REPEAT;
    let focus = imgui::ui_get_focus();
    match s.screen {
        Screen::About => {
            if (key == input::KEY_BACK || key == input::KEY_LEFT) && short {
                s.screen = Screen::Menu;
                STATE.set(s);
                api::request_render();
                return;
            }
        }
        Screen::Keyboard => {
            if key == input::KEY_BACK && long {
                s.screen = Screen::Wifi;
                imgui::ui_set_focus(0);
                STATE.set(s);
                api::request_render();
                return;
            }
        }
        Screen::Network => {
            if key == input::KEY_BACK && short {
                s.screen = Screen::Wifi;
                imgui::ui_set_focus(0);
                STATE.set(s);
                api::request_render();
                return;
            }
        }
        Screen::Wifi => {
            if key == input::KEY_BACK && short {
                s.screen = Screen::Menu;
                imgui::ui_set_focus(ITEM_WIFI);
                STATE.set(s);
                api::request_render();
                return;
            }
            let toggle = matches!(key, input::KEY_LEFT | input::KEY_RIGHT | input::KEY_OK);
            if focus == WIFI_TOGGLE && toggle && short {
                wifi::set_enabled(!wifi::enabled());
            }
        }
        Screen::Menu => {
            if key == input::KEY_BACK && short {
                api::exit_to_launcher();
                return;
            }
            match (focus, key) {
                (ITEM_BRIGHTNESS, input::KEY_LEFT) if step => {
                    s.brightness = s.brightness.saturating_sub(10).max(10);
                    api::settings_set_u32(NS, KEY_BRIGHTNESS, s.brightness);
                }
                (ITEM_BRIGHTNESS, input::KEY_RIGHT) if step => {
                    s.brightness = (s.brightness + 10).min(100);
                    api::settings_set_u32(NS, KEY_BRIGHTNESS, s.brightness);
                }
                (ITEM_SOUND, input::KEY_LEFT) | (ITEM_SOUND, input::KEY_RIGHT) | (ITEM_SOUND, input::KEY_OK)
                    if short =>
                {
                    s.sound = !s.sound;
                    api::settings_set_u32(NS, KEY_SOUND, s.sound as u32);
                }
                _ => {}
            }
        }
    }
    STATE.set(s);
    imgui::ui_input(key as u8, kind as u8);
}

// -- helpers --------------------------------------------------------------------------

/// First `max` characters, on a char boundary.
fn clip(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Fixed-capacity label builder. No allocation.
struct Small {
    buf: [u8; 40],
    len: usize,
}

impl Small {
    const fn new() -> Self {
        Self { buf: [0; 40], len: 0 }
    }
    fn push_str(&mut self, s: &str) {
        for &b in s.as_bytes() {
            if self.len < self.buf.len() {
                self.buf[self.len] = b;
                self.len += 1;
            }
        }
    }
    fn push(&mut self, mut n: u32) {
        let mut digits = [0u8; 10];
        let mut i = 0;
        loop {
            digits[i] = b'0' + (n % 10) as u8;
            n /= 10;
            i += 1;
            if n == 0 {
                break;
            }
        }
        while i > 0 {
            i -= 1;
            if self.len < self.buf.len() {
                self.buf[self.len] = digits[i];
                self.len += 1;
            }
        }
    }
    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

api::export_render!(render_impl);
api::export_on_input!(on_input_impl);
api::export_on_start!(on_start_impl);
api::wasm_panic_handler!();
