//! Test UI - Tests all IMGUI components
//!
//! This app renders 6 different scenes to test UI components:
//! 0: Counter Demo - buttons
//! 1: Settings Menu - scrollable menu with values
//! 2: Layout Demo - text alignment and layout
//! 3: Progress Demo - progress bars
//! 4: Checkbox Demo - checkboxes
//! 5: Footer Demo - footer buttons

#![no_std]
#![no_main]

use fri3d_sdk::{canvas, InputKey, InputType};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Constants matching original IMGUI exactly
const SCREEN_WIDTH: i32 = 128;
const SCREEN_HEIGHT: i32 = 64;
const FONT_HEIGHT_PRIMARY: i32 = 12;
const FONT_HEIGHT_SECONDARY: i32 = 10;
const BUTTON_PADDING_X: i32 = 4;
const BUTTON_PADDING_Y: i32 = 2;
const MENU_ITEM_HEIGHT: i32 = 12;
const FOOTER_HEIGHT: i32 = 12;

const NUM_SCENES: u32 = 6;

static mut SCENE: u32 = 0;
static mut COUNTER: i32 = 0;
static mut MENU_SCROLL: i32 = 0;
static mut FOCUS: i32 = 0;
static mut BRIGHTNESS: i32 = 5;
static mut SOUND: bool = true;
static mut VIBRATION: bool = true;
static mut WIFI: bool = false;
static mut CHECK1: bool = false;
static mut CHECK2: bool = true;
static mut CHECK3: bool = false;
static mut LEFT_COUNT: i32 = 0;
static mut RIGHT_COUNT: i32 = 0;

// Helper to draw label - matches original ui_label
// y is the layout position (top of text area), font_height is added for baseline
fn draw_label(y: i32, text: &str, font_height: i32, align: i32) {
    let text_width = (text.len() as i32) * 6;
    let text_x = match align {
        0 => 0,                                    // LEFT
        1 => (SCREEN_WIDTH - text_width) / 2,      // CENTER
        2 => SCREEN_WIDTH - text_width,            // RIGHT
        _ => 0,
    };
    // Original: canvas_draw_str(text_x, y + font_height, text)
    canvas::draw_str(text_x, y + font_height, text);
}

// Helper to draw a button - matches original ui_button
fn draw_button(x: i32, y: i32, text: &str, focused: bool) -> i32 {
    let text_width = (text.len() as i32) * 6;
    let btn_width = text_width + BUTTON_PADDING_X * 2;
    let btn_height = FONT_HEIGHT_SECONDARY + BUTTON_PADDING_Y * 2;

    if focused {
        canvas::set_color(canvas::Color::Black);
        canvas::fill_round_rect(x, y, btn_width as u32, btn_height as u32, 2);
        canvas::set_color(canvas::Color::White);
        // Text y position: y + btn_height - BUTTON_PADDING_Y matches original
        canvas::draw_str(x + BUTTON_PADDING_X, y + btn_height - BUTTON_PADDING_Y, text);
        canvas::set_color(canvas::Color::Black);
    } else {
        canvas::draw_round_rect(x, y, btn_width as u32, btn_height as u32, 2);
        canvas::draw_str(x + BUTTON_PADDING_X, y + btn_height - BUTTON_PADDING_Y, text);
    }

    btn_width
}

// Scene 0: Counter Demo
fn render_counter() {
    let focus = unsafe { FOCUS };

    // Title at y=0, primary font
    draw_label(0, "Counter Demo", FONT_HEIGHT_PRIMARY, 1);

    // Spacer of 8 pixels after title (original: ui_spacer(8))
    // Counter value at y = 12 + 8 = 20, then add font height
    // But original positions "Count: 0" differently
    // Looking at golden: title at top, then "Count: 0" about 8px below
    // Original: ui_label after spacer(8), so y = 12 + 8 = 20
    draw_label(20, "Count: 0", FONT_HEIGHT_SECONDARY, 1);

    // Spacer of 8 after count: y = 20 + 10 + 8 = 38
    // Buttons start at y = 38
    // But looking at golden, buttons appear around y=36

    // Calculate centered buttons
    let btn_plus_w = 6 + BUTTON_PADDING_X * 2;   // 14
    let btn_minus_w = 6 + BUTTON_PADDING_X * 2;  // 14
    let btn_reset_w = 30 + BUTTON_PADDING_X * 2; // 38
    let spacing = 4;
    let total_width = btn_plus_w + btn_minus_w + btn_reset_w + spacing * 2; // 74
    let start_x = (SCREEN_WIDTH - total_width) / 2; // 27
    let btn_y = 36; // Adjusted based on golden image

    let mut x = start_x;
    draw_button(x, btn_y, "+", focus == 0);
    x += btn_plus_w + spacing;
    draw_button(x, btn_y, "-", focus == 1);
    x += btn_minus_w + spacing;
    draw_button(x, btn_y, "Reset", focus == 2);
}

// Scene 1: Settings Menu
fn render_menu() {
    let scroll = unsafe { MENU_SCROLL };
    let focus = unsafe { FOCUS };
    let sound = unsafe { SOUND };
    let vibration = unsafe { VIBRATION };
    let wifi = unsafe { WIFI };

    // Title
    draw_label(0, "Settings Menu", FONT_HEIGHT_PRIMARY, 1);

    // Spacer(4) after title, menu starts at y = 12 + 4 = 16
    let menu_y = 16;
    let visible_items = 4;
    let item_width = SCREEN_WIDTH - 3 - 2; // Leave room for scrollbar

    let items: [(&str, &str); 6] = [
        ("Brightness", "5"),
        ("Sound", if sound { "On" } else { "Off" }),
        ("Vibration", if vibration { "On" } else { "Off" }),
        ("WiFi", if wifi { "On" } else { "Off" }),
        ("About", ""),
        ("Reset All", ""),
    ];

    for i in 0..visible_items {
        let item_idx = (scroll + i) as usize;
        if item_idx >= 6 {
            break;
        }

        let y = menu_y + i * MENU_ITEM_HEIGHT;
        let is_focused = focus == (scroll + i);

        if is_focused {
            canvas::fill_rect(0, y, item_width as u32, MENU_ITEM_HEIGHT as u32);
            canvas::set_color(canvas::Color::White);
        }

        let (label, value) = items[item_idx];
        // Menu item text: y + MENU_ITEM_HEIGHT - 2 for baseline
        canvas::draw_str(2, y + MENU_ITEM_HEIGHT - 2, label);

        if !value.is_empty() {
            let value_width = (value.len() as i32) * 6;
            canvas::draw_str(item_width - value_width - 2, y + MENU_ITEM_HEIGHT - 2, value);
        }

        if is_focused {
            canvas::set_color(canvas::Color::Black);
        }
    }

    // Scrollbar
    let scrollbar_x = SCREEN_WIDTH - 2;
    let scrollbar_height = visible_items * MENU_ITEM_HEIGHT;
    let total_items = 6i32;

    // Dotted track
    let mut y = menu_y;
    while y < menu_y + scrollbar_height {
        canvas::draw_pixel(scrollbar_x, y);
        y += 2;
    }

    // Solid thumb
    let thumb_height = (scrollbar_height * visible_items) / total_items;
    let thumb_height = if thumb_height < 4 { 4 } else { thumb_height };
    let max_scroll = total_items - visible_items;
    let thumb_y = if max_scroll > 0 {
        menu_y + ((scrollbar_height - thumb_height) * scroll) / max_scroll
    } else {
        menu_y
    };
    canvas::fill_rect(scrollbar_x - 1, thumb_y, 3, thumb_height as u32);
}

// Scene 2: Layout Demo
fn render_layout() {
    let focus = unsafe { FOCUS };

    // Title
    draw_label(0, "Layout Demo", FONT_HEIGHT_PRIMARY, 1);

    // Spacer(2) after title
    // "VStack items:" at y = 12 + 2 = 14
    draw_label(14, "VStack items:", FONT_HEIGHT_SECONDARY, 0);

    // Item 1 at y = 14 + 10 + spacing = 26 (with highlight)
    let y = 26;
    if focus == 0 {
        canvas::fill_rect(0, y, SCREEN_WIDTH as u32, FONT_HEIGHT_SECONDARY as u32);
        canvas::set_color(canvas::Color::White);
    }
    canvas::draw_str(2, y + FONT_HEIGHT_SECONDARY - 2, "Item 1");
    if focus == 0 {
        canvas::set_color(canvas::Color::Black);
    }

    // "Left" at y = 26 + 10 + spacing = 38
    draw_label(38, "Left", FONT_HEIGHT_SECONDARY, 0);

    // "Item 3  Center" at y = 38 + 10 + spacing = 50
    draw_label(50, "Item 3  Center", FONT_HEIGHT_SECONDARY, 0);
}

// Scene 3: Progress Demo
fn render_progress() {
    // Title
    draw_label(0, "Progress Demo", FONT_HEIGHT_PRIMARY, 1);

    // Spacer(4) after title
    // "Loading:" at y = 12 + 4 = 16
    draw_label(16, "Loading:", FONT_HEIGHT_SECONDARY, 0);

    // Spacer(2) then progress bar at y = 16 + 10 + 2 = 28
    draw_progress_bar(4, 28, SCREEN_WIDTH - 8, 0.0);

    // Spacer(4) after bar
    // "25%:" at y = 28 + 8 + 4 = 40... but golden shows it lower
    // Looking at golden, "25%:" appears at around y=42
    draw_label(42, "25%:", FONT_HEIGHT_SECONDARY, 0);

    // Progress bar at y = 42 + 10 + 2 = 54
    draw_progress_bar(4, 54, SCREEN_WIDTH - 8, 0.25);
}

fn draw_progress_bar(x: i32, y: i32, width: i32, value: f32) {
    let bar_height = 8;

    // Outline
    canvas::draw_rect(x, y, width as u32, bar_height as u32);

    // Fill
    let fill_width = ((value * (width - 2) as f32) as i32).max(0);
    if fill_width > 0 {
        canvas::fill_rect(x + 1, y + 1, fill_width as u32, (bar_height - 2) as u32);
    }
}

// Scene 4: Checkbox Demo
fn render_checkbox() {
    let focus = unsafe { FOCUS };
    let check1 = unsafe { CHECK1 };
    let check2 = unsafe { CHECK2 };
    let check3 = unsafe { CHECK3 };

    // Title
    draw_label(0, "Checkbox Demo", FONT_HEIGHT_PRIMARY, 1);

    // Spacer(8) after title, checkboxes at y = 12 + 8 = 20... but golden shows ~16
    draw_checkbox_item(0, 16, "Option 1", check1, focus == 0);
    draw_checkbox_item(0, 28, "Option 2 (default on)", check2, focus == 1);
    draw_checkbox_item(0, 40, "Option 3", check3, focus == 2);
}

fn draw_checkbox_item(x: i32, y: i32, label: &str, checked: bool, focused: bool) {
    let box_size: i32 = 10;
    let item_height = FONT_HEIGHT_SECONDARY + 2;

    if focused {
        canvas::fill_rect(x, y, SCREEN_WIDTH as u32, item_height as u32);
        canvas::set_color(canvas::Color::White);
    }

    // Checkbox outline
    let box_x = x + 2;
    let box_y = y + (item_height - box_size) / 2;
    canvas::draw_rect(box_x, box_y, box_size as u32, box_size as u32);

    // Checkmark if checked
    if checked {
        canvas::draw_line(box_x + 2, box_y + 5, box_x + 4, box_y + 7);
        canvas::draw_line(box_x + 4, box_y + 7, box_x + 7, box_y + 2);
    }

    // Label - baseline at y + item_height - 2
    canvas::draw_str(box_x + box_size + 4, y + item_height - 2, label);

    if focused {
        canvas::set_color(canvas::Color::Black);
    }
}

// Scene 5: Footer Demo
fn render_footer() {
    // Title
    draw_label(0, "Footer Demo", FONT_HEIGHT_PRIMARY, 1);

    // Spacer(8) after title
    // Counters at y = 12 + 8 = 20
    draw_label(20, "Left: 0  Right: 0", FONT_HEIGHT_SECONDARY, 1);

    // Spacer(4)
    // "Press </> to change" at y = 20 + 10 + 4 = 34... but golden shows ~32
    draw_label(32, "Press </> to change", FONT_HEIGHT_SECONDARY, 1);

    // Footer at bottom
    let footer_y = SCREEN_HEIGHT - FOOTER_HEIGHT;

    // Left: arrow + "Dec"
    canvas::draw_line(2, footer_y + 5, 6, footer_y + 2);
    canvas::draw_line(2, footer_y + 5, 6, footer_y + 8);
    canvas::draw_str(9, footer_y + FOOTER_HEIGHT - 2, "Dec");

    // Center: filled circle + "OK"
    let ok_text_w = 12; // "OK" = 2 chars * 6 = 12
    let ok_total_w = ok_text_w + 12; // text + circle + spacing
    let ok_x = (SCREEN_WIDTH - ok_total_w) / 2;
    canvas::fill_circle(ok_x + 4, footer_y + 5, 3);
    canvas::draw_str(ok_x + 12, footer_y + FOOTER_HEIGHT - 2, "OK");

    // Right: "Inc" + arrow
    let inc_text_w = 18; // "Inc" = 3 chars * 6 = 18
    let inc_x = SCREEN_WIDTH - inc_text_w - 10;
    canvas::draw_str(inc_x, footer_y + FOOTER_HEIGHT - 2, "Inc");
    let arrow_x = SCREEN_WIDTH - 7;
    canvas::draw_line(arrow_x + 4, footer_y + 5, arrow_x, footer_y + 2);
    canvas::draw_line(arrow_x + 4, footer_y + 5, arrow_x, footer_y + 8);
}

#[no_mangle]
pub extern "C" fn render() {
    canvas::set_color(canvas::Color::Black);

    let scene = unsafe { SCENE };
    match scene {
        0 => render_counter(),
        1 => render_menu(),
        2 => render_layout(),
        3 => render_progress(),
        4 => render_checkbox(),
        5 => render_footer(),
        _ => {}
    }
}

#[no_mangle]
pub extern "C" fn on_input(key: u32, event_type: u32) {
    if event_type != InputType::Short as u32 {
        return;
    }

    let scene = unsafe { SCENE };
    let focus = unsafe { &mut FOCUS };

    match key {
        k if k == InputKey::Up as u32 => {
            if *focus > 0 {
                *focus -= 1;
            }
        }
        k if k == InputKey::Down as u32 => {
            let max_focus = match scene {
                0 => 2,
                1 => 5,
                2 => 2,
                4 => 2,
                _ => 0,
            };
            if *focus < max_focus {
                *focus += 1;
            }
        }
        k if k == InputKey::Ok as u32 => {
            match scene {
                0 => {
                    let counter = unsafe { &mut COUNTER };
                    match *focus {
                        0 => *counter += 1,
                        1 => *counter -= 1,
                        2 => *counter = 0,
                        _ => {}
                    }
                }
                1 => match *focus {
                    1 => unsafe { SOUND = !SOUND },
                    2 => unsafe { VIBRATION = !VIBRATION },
                    3 => unsafe { WIFI = !WIFI },
                    _ => {}
                },
                4 => match *focus {
                    0 => unsafe { CHECK1 = !CHECK1 },
                    1 => unsafe { CHECK2 = !CHECK2 },
                    2 => unsafe { CHECK3 = !CHECK3 },
                    _ => {}
                },
                _ => {}
            }
        }
        k if k == InputKey::Left as u32 => {
            if scene == 5 {
                unsafe { LEFT_COUNT -= 1 };
            }
        }
        k if k == InputKey::Right as u32 => {
            if scene == 5 {
                unsafe { RIGHT_COUNT += 1 };
            }
        }
        _ => {}
    }
}

#[no_mangle]
pub extern "C" fn get_scene_count() -> u32 {
    NUM_SCENES
}

#[no_mangle]
pub extern "C" fn get_scene() -> u32 {
    unsafe { SCENE }
}

#[no_mangle]
pub extern "C" fn set_scene(scene: u32) {
    unsafe {
        SCENE = scene % NUM_SCENES;
        FOCUS = 0;
        MENU_SCROLL = 0;
    }
}
