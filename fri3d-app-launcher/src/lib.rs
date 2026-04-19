#![no_std]
#![deny(unsafe_code)]

// Fri3d Camp Badge — Launcher OS.
// Implements the design system from `fri3d-camp-badge-ui/project/index.html`:
//   - Idle screen (default boot state) — animated geometric fox, cycles
//     through SLEEP / STARS / HACK scenes. Any button wakes it.
//   - App Grid — 4x2 vector-icon launcher. D-pad navigates, A launches.
//
// All rendering goes through the new screen_* host imports (full 296x240
// RGB565). No legacy mono canvas use here — this is the showcase for the
// new color pipeline.

use fri3d_wasm_api as api;
use api::{screen, theme};
use api::input as kbd;

const W: i32 = 296;
const H: i32 = 240;

// ---- App table -------------------------------------------------------------
// IDs match firmware/scripts/embed_apps.sh. The launcher itself is id=0
// (special — never appears in the grid).
struct App {
    id: u32,
    name: &'static str,
    icon: fn(cx: i32, cy: i32, color: u32),
}

const APPS: &[App] = &[
    App { id: 1, name: "CIRCLES", icon: icon_circles },
    App { id: 5, name: "SNAKE",   icon: icon_snake   },
    App { id: 2, name: "FRACTAL", icon: icon_fractal },
    App { id: 6, name: "WIFIPET", icon: icon_wifi    },
    App { id: 3, name: "DRAW",    icon: icon_draw    },
    App { id: 4, name: "UI TEST", icon: icon_ui      },
];

// ---- State -----------------------------------------------------------------
// 0 = IDLE, 1 = APP GRID. Kept as i16 because AppCell only takes Copy.
const SCENE_IDLE: i16 = 0;
const SCENE_GRID: i16 = 1;

static SCENE: api::AppCell<i16>     = api::AppCell::new(SCENE_IDLE);
static SELECTED: api::AppCell<i16>  = api::AppCell::new(0);
// Idle phase rolls 0..3 every ~3s (driven off get_time_ms in render).
static IDLE_PHASE: api::AppCell<i16> = api::AppCell::new(0);

// ---- Render ----------------------------------------------------------------
fn render_impl() {
    // Idle scene self-cycles every ~3s of wall clock so the badge looks
    // alive at rest. Uses get_time_ms so it doesn't depend on render
    // cadence — works the same on hardware (30fps) and the browser harness.
    let phase = ((api::get_time_ms() / 3000) % 3) as i16;
    IDLE_PHASE.set(phase);

    screen::clear(theme::BG);

    match SCENE.get() {
        SCENE_GRID => draw_app_grid(),
        _          => draw_idle(phase),
    }
}

// ---- Input -----------------------------------------------------------------
fn on_input_impl(key: u32, kind: u32) {
    if kind != kbd::TYPE_SHORT_PRESS {
        return;
    }

    match SCENE.get() {
        SCENE_IDLE => {
            // Any button wakes the badge into the app grid.
            SCENE.set(SCENE_GRID);
        }
        SCENE_GRID => {
            let mut sel = SELECTED.get();
            let count = APPS.len() as i16;
            match key {
                kbd::KEY_LEFT  => { sel = (sel + count - 1) % count; }
                kbd::KEY_RIGHT => { sel = (sel + 1)         % count; }
                kbd::KEY_UP => {
                    sel = if sel >= 3 { sel - 3 } else { sel };
                }
                kbd::KEY_DOWN => {
                    sel = if sel + 3 < count { sel + 3 } else { sel };
                }
                kbd::KEY_OK => {
                    api::start_app(APPS[sel as usize].id);
                }
                kbd::KEY_BACK => {
                    SCENE.set(SCENE_IDLE);
                }
                _ => {}
            }
            SELECTED.set(sel);
        }
        _ => {}
    }
}

// ---- Idle scene ------------------------------------------------------------
fn draw_idle(phase: i16) {
    draw_status_bar("IDLE", "03:14");

    match phase {
        0 => scene_sleep(),
        1 => scene_stars(),
        _ => scene_hack(),
    }

    // Bottom info strip — matches the design's bottom band.
    let y_strip = H - 22;
    screen::line(0, y_strip, W, y_strip, theme::INK_DEEP);
    screen::text(6, H - 12, "FRI3D CAMP '26", theme::WHITE, api::font::PRIMARY);

    let label = match phase {
        0 => "SCENE - SLEEP",
        1 => "SCENE - STARS",
        _ => "SCENE - HACK",
    };
    screen::text(W / 2 - 35, H - 12, label, theme::INK_DIM, api::font::PRIMARY);
    screen::text(W - 70, H - 12, "PRESS A", theme::ACCENT, api::font::PRIMARY);
    screen::text(6, H - 4, "@ruisender ID #0742", theme::INK_DIM, api::font::PRIMARY);
}

fn scene_sleep() {
    // Horizon
    screen::line(0, 170, W, 170, theme::INK_DEEP);
    // Moon
    screen::circle(240, 50, 14, theme::INK);
    screen::circle(236, 46, 2,  theme::INK_DEEP);
    screen::circle(244, 54, 1,  theme::INK_DEEP);
    // Stars (cross marks)
    let stars: &[(i32, i32)] = &[(40, 30), (80, 50), (120, 25), (180, 60), (210, 20), (260, 90)];
    for &(x, y) in stars {
        screen::line(x - 2, y, x + 2, y, theme::INK);
        screen::line(x, y - 2, x, y + 2, theme::INK);
    }
    // Curled fox (geometric) at (100, 140)
    let fx = 100;
    let fy = 140;
    // Body — approximated as four lines forming a curved silhouette
    screen::line(fx,      fy + 20, fx + 15, fy - 5,  theme::INK);
    screen::line(fx + 15, fy - 5,  fx + 50, fy - 8,  theme::INK);
    screen::line(fx + 50, fy - 8,  fx + 75, fy - 5,  theme::INK);
    screen::line(fx + 75, fy - 5,  fx + 85, fy + 15, theme::INK);
    screen::line(fx,      fy + 20, fx + 85, fy + 28, theme::INK);
    screen::line(fx + 85, fy + 15, fx + 85, fy + 28, theme::INK);
    // Head — diamond tucked in
    fox_head(fx + 60, fy, theme::INK, /*sleeping=*/ true);
    // Tail curl
    screen::line(fx,      fy + 20, fx - 8, fy + 10, theme::INK);
    screen::line(fx - 8,  fy + 10, fx - 4, fy - 2,  theme::INK);
    screen::line(fx - 4,  fy - 2,  fx + 4, fy - 8,  theme::INK);
    // Zzz
    screen::text(190, 110, "z", theme::ACCENT, api::font::SECONDARY);
    screen::text(200, 100, "z", theme::ACCENT, api::font::PRIMARY);
    screen::text(208, 92,  "z", theme::ACCENT, api::font::PRIMARY);
}

fn scene_stars() {
    // Hills
    let hills: &[(i32, i32)] = &[
        (0, 175), (30, 160), (70, 170), (110, 155), (150, 168),
        (190, 150), (230, 165), (296, 158),
    ];
    for w in hills.windows(2) {
        screen::line(w[0].0, w[0].1, w[1].0, w[1].1, theme::INK_DEEP);
    }
    screen::line(0, 175, W, 175, theme::INK_DEEP);
    // Constellation — the fox itself (connect-the-dots)
    let c: &[(i32, i32)] = &[
        (60, 40), (90, 35), (120, 50), (105, 65), (135, 70), (80, 55), (145, 45),
    ];
    for &(x, y) in c {
        screen::disc(x, y, 1, theme::WHITE);
    }
    let lines: &[(usize, usize)] = &[(0, 1), (1, 5), (5, 3), (3, 4), (4, 2), (2, 6), (6, 1)];
    for &(a, b) in lines {
        screen::line(c[a].0, c[a].1, c[b].0, c[b].1, theme::INK_DEEP);
    }
    // Scattered stars
    let stars: &[(i32, i32)] = &[(20, 25), (180, 30), (220, 55), (260, 40), (200, 80), (40, 75)];
    for &(x, y) in stars {
        screen::pixel(x, y, theme::WHITE);
    }
    // Shooting star
    screen::line(230, 25, 260, 45, theme::ACCENT);
    // Fox sitting, looking up
    let fx = 150;
    let fy = 145;
    // Body
    screen::line(fx,      fy + 20, fx,      fy,      theme::INK);
    screen::line(fx,      fy,      fx + 12, fy,      theme::INK);
    screen::line(fx + 12, fy,      fx + 28, fy,      theme::INK);
    screen::line(fx + 28, fy,      fx + 40, fy,      theme::INK);
    screen::line(fx + 40, fy,      fx + 40, fy + 20, theme::INK);
    screen::line(fx,      fy + 25, fx + 40, fy + 25, theme::INK);
    screen::line(fx + 40, fy + 20, fx + 40, fy + 25, theme::INK);
    screen::line(fx,      fy + 20, fx,      fy + 25, theme::INK);
    fox_head(fx + 10, fy - 5, theme::INK, false);
    // Tail
    screen::line(fx + 40, fy + 20, fx + 55, fy + 10, theme::INK);
    screen::line(fx + 55, fy + 10, fx + 58, fy - 5,  theme::INK);
}

fn scene_hack() {
    // Desk line
    screen::line(0, 175, W, 175, theme::INK_DEEP);
    // Terminal frame
    screen::stroke_rect(28, 40, 130, 90, theme::INK);
    screen::line(28, 50, 158, 50, theme::INK_DEEP);
    screen::disc(33, 45, 1, theme::INK_DEEP);
    screen::disc(39, 45, 1, theme::INK_DEEP);
    screen::disc(45, 45, 1, theme::INK_DEEP);
    screen::text(34, 62, "$ sudo foxctl --pwn", theme::ACCENT, api::font::PRIMARY);
    screen::text(34, 72, "scanning mesh...",    theme::INK_DIM, api::font::PRIMARY);
    screen::text(34, 82, "found 142 badges",    theme::INK_DIM, api::font::PRIMARY);
    screen::text(34, 92, "handshake ok",        theme::ACCENT, api::font::PRIMARY);
    screen::text(34, 102, "######... 68%",       theme::WHITE,  api::font::PRIMARY);
    screen::text(34, 112, "injecting..._",       theme::INK_DIM, api::font::PRIMARY);
    screen::text(34, 122, "$ _",                 theme::ACCENT, api::font::PRIMARY);
    // Fox at keyboard at (180, 105)
    let fx = 180;
    let fy = 105;
    fox_head(fx, fy, theme::INK, false);
    // Body
    screen::stroke_rect(fx - 10, fy + 30, 48, 25, theme::INK);
    // Paws on keyboard
    screen::stroke_rect(fx - 2, fy + 50, 6, 5, theme::INK);
    screen::stroke_rect(fx + 24, fy + 50, 6, 5, theme::INK);
    // Keyboard
    screen::line(170, 158, 250, 158, theme::INK_DEEP);
    screen::line(170, 162, 250, 162, theme::INK_DEEP);
}

// Geometric Fox head (matches design's FoxMark): inverted diamond face
// with two triangular ears. `sleeping` swaps eyes for closed lids.
fn fox_head(x: i32, y: i32, color: u32, sleeping: bool) {
    // Face: inverted diamond
    screen::line(x,      y,      x + 28, y,      color);
    screen::line(x,      y,      x + 14, y + 22, color);
    screen::line(x + 28, y,      x + 14, y + 22, color);
    // Snout detail
    screen::line(x,      y,      x + 14, y + 10, color);
    screen::line(x + 28, y,      x + 14, y + 10, color);
    screen::line(x + 14, y + 10, x + 14, y + 22, color);
    // Ears
    screen::line(x,      y,      x + 4,  y - 8,  color);
    screen::line(x + 4,  y - 8,  x + 10, y,      color);
    screen::line(x + 28, y,      x + 24, y - 8,  color);
    screen::line(x + 24, y - 8,  x + 18, y,      color);
    // Eyes
    if sleeping {
        screen::line(x + 5,  y + 6, x + 9,  y + 6, theme::WHITE);
        screen::line(x + 19, y + 6, x + 23, y + 6, theme::WHITE);
    } else {
        screen::pixel(x + 7,  y + 6, theme::WHITE);
        screen::pixel(x + 21, y + 6, theme::WHITE);
        screen::pixel(x + 7,  y + 7, theme::WHITE);
        screen::pixel(x + 21, y + 7, theme::WHITE);
    }
}

// ---- App grid scene --------------------------------------------------------
fn draw_app_grid() {
    draw_status_bar("APPS", "14:32");

    screen::text(W / 2 - 25, 22, "SELECT APP", theme::WHITE, api::font::PRIMARY);

    let cols: i32 = 3;
    let cell_w: i32 = (W - 20) / cols;     // ~92
    let cell_h: i32 = 86;
    let start_y: i32 = 32;

    let count = APPS.len() as i32;
    let sel = SELECTED.get() as i32;

    for i in 0..count {
        let col = i % cols;
        let row = i / cols;
        let cx = 10 + col * cell_w + 4;
        let cy = start_y + row * cell_h;
        let inner_w = cell_w - 8;
        let inner_h = cell_h - 8;
        let is_sel = i == sel;

        let border = if is_sel { theme::ACCENT } else { theme::INK_DEEP };
        screen::stroke_rect(cx, cy, inner_w, inner_h, border);

        // Selection: dashed outer accent ring (we draw four small ticks
        // at each side rather than full dashes — keeps it cheap and
        // visually distinct).
        if is_sel {
            let oc = cx - 3;
            let oy = cy - 3;
            let ow = inner_w + 6;
            let oh = inner_h + 6;
            // Corner brackets
            let l = 6;
            screen::line(oc,         oy,         oc + l,     oy,         theme::ACCENT);
            screen::line(oc,         oy,         oc,         oy + l,     theme::ACCENT);
            screen::line(oc + ow,    oy,         oc + ow - l, oy,        theme::ACCENT);
            screen::line(oc + ow,    oy,         oc + ow,    oy + l,     theme::ACCENT);
            screen::line(oc,         oy + oh,    oc + l,     oy + oh,    theme::ACCENT);
            screen::line(oc,         oy + oh,    oc,         oy + oh - l, theme::ACCENT);
            screen::line(oc + ow,    oy + oh,    oc + ow - l, oy + oh,   theme::ACCENT);
            screen::line(oc + ow,    oy + oh,    oc + ow,    oy + oh - l, theme::ACCENT);
        }

        // Icon centred in upper portion of the cell
        let icon_color = if is_sel { theme::ACCENT } else { theme::INK };
        (APPS[i as usize].icon)(cx + inner_w / 2, cy + 28, icon_color);

        // Name centred near bottom
        let label = APPS[i as usize].name;
        let label_w = (label.len() as i32) * 6;  // approximate
        let lx = cx + (inner_w - label_w) / 2;
        let ly = cy + inner_h - 12;
        let label_color = if is_sel { theme::ACCENT } else { theme::WHITE };
        screen::text(lx, ly, label, label_color, api::font::PRIMARY);
    }

    // Bottom hint bar
    let y_strip = H - 14;
    screen::line(0, y_strip, W, y_strip, theme::INK_DEEP);
    screen::text(6, H - 5, "NAV: ARROWS  A: LAUNCH  B: BACK", theme::INK_DIM, api::font::PRIMARY);
}

// ---- Status bar ------------------------------------------------------------
fn draw_status_bar(label: &str, time: &str) {
    screen::line(0, 14, W, 14, theme::INK_DEEP);
    screen::text(6, 10, label, theme::WHITE, api::font::PRIMARY);
    let tx = W - 30;
    screen::text(tx, 10, time, theme::INK, api::font::PRIMARY);
    // Battery icon
    let bx = W - 60;
    screen::stroke_rect(bx, 4, 18, 8, theme::INK);
    screen::fill_rect(bx + 18, 6, 1, 4, theme::INK);
    screen::fill_rect(bx + 2, 6, 11, 4, theme::INK);
    // Wifi icon (3 arcs as concentric small circles cut to top half)
    let wx = W - 78;
    screen::pixel(wx + 4, 10, theme::INK);
    screen::pixel(wx + 3, 10, theme::INK);
    screen::pixel(wx + 5, 10, theme::INK);
    screen::pixel(wx + 4, 9,  theme::INK);
    screen::pixel(wx + 3, 8,  theme::INK);
    screen::pixel(wx + 5, 8,  theme::INK);
    screen::pixel(wx + 2, 7,  theme::INK);
    screen::pixel(wx + 6, 7,  theme::INK);
}

// ---- App icons (vector glyphs in ~20x20 box centred on cx,cy) --------------
fn icon_circles(cx: i32, cy: i32, color: u32) {
    screen::circle(cx, cy, 4,  color);
    screen::circle(cx, cy, 8,  color);
    screen::circle(cx, cy, 12, color);
}

fn icon_snake(cx: i32, cy: i32, color: u32) {
    // Stepped path
    screen::line(cx - 9, cy - 6, cx - 3, cy - 6, color);
    screen::line(cx - 3, cy - 6, cx - 3, cy,     color);
    screen::line(cx - 3, cy,     cx + 3, cy,     color);
    screen::line(cx + 3, cy,     cx + 3, cy + 6, color);
    screen::line(cx + 3, cy + 6, cx + 9, cy + 6, color);
    // Head + apple
    screen::fill_rect(cx - 10, cy - 7, 2, 2, color);
    screen::fill_rect(cx + 8,  cy + 5, 2, 2, theme::ACCENT);
}

fn icon_fractal(cx: i32, cy: i32, color: u32) {
    // Mandelbrot-ish "M" outline
    screen::stroke_rect(cx - 8, cy - 8, 16, 16, color);
    screen::line(cx - 8, cy + 8, cx,     cy - 8, color);
    screen::line(cx,     cy - 8, cx + 8, cy + 8, color);
    screen::pixel(cx, cy, color);
}

fn icon_wifi(cx: i32, cy: i32, color: u32) {
    // Three rising arcs
    screen::circle(cx, cy + 10, 9, color);
    screen::circle(cx, cy + 10, 6, color);
    screen::circle(cx, cy + 10, 3, color);
    screen::disc(cx, cy + 10, 1, color);
    // Mask out the bottom half by overlaying background hlines (cheap arc fake)
    for y in (cy + 11)..(cy + 20) {
        screen::line(cx - 12, y, cx + 12, y, theme::BG);
    }
}

fn icon_draw(cx: i32, cy: i32, color: u32) {
    // Pencil-like glyph
    screen::stroke_rect(cx - 8, cy - 4, 14, 8, color);
    screen::line(cx + 6, cy - 4, cx + 10, cy,     color);
    screen::line(cx + 6, cy + 4, cx + 10, cy,     color);
    // Squiggle line beneath
    screen::line(cx - 8, cy + 8, cx - 4, cy + 6,  color);
    screen::line(cx - 4, cy + 6, cx,     cy + 8,  color);
    screen::line(cx,     cy + 8, cx + 4, cy + 6,  color);
    screen::line(cx + 4, cy + 6, cx + 8, cy + 8,  color);
}

fn icon_ui(cx: i32, cy: i32, color: u32) {
    // Window with title bar + a button
    screen::stroke_rect(cx - 9, cy - 8, 18, 16, color);
    screen::line(cx - 9, cy - 4, cx + 9, cy - 4, color);
    screen::fill_rect(cx - 6, cy + 1, 6, 4, color);
    screen::stroke_rect(cx + 1, cy + 1, 6, 4, color);
}

api::export_render!(render_impl);
api::export_on_input!(on_input_impl);
api::wasm_panic_handler!();
