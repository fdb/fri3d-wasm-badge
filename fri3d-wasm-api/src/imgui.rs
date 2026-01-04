#![allow(clippy::cast_possible_truncation)]

use crate::{canvas_draw_box, canvas_draw_disc, canvas_draw_dot, canvas_draw_frame,
            canvas_draw_line, canvas_draw_rbox, canvas_draw_rframe, canvas_draw_str,
            canvas_set_color, canvas_set_font, canvas_string_width, canvas_clear};
use crate::{align, color, font, input};

const UI_SCREEN_WIDTH: i16 = 128;
const UI_SCREEN_HEIGHT: i16 = 64;

const UI_MAX_LAYOUT_DEPTH: usize = 8;
const UI_MAX_FOCUSABLE: i16 = 32;
const UI_MAX_DEFERRED: usize = 16;
const UI_DEFERRED_TEXT_CAP: usize = 128;
const UI_FONT_HEIGHT_PRIMARY: i16 = 12;
const UI_FONT_HEIGHT_SECONDARY: i16 = 11;
const UI_BUTTON_PADDING_X: i16 = 4;
const UI_BUTTON_PADDING_Y: i16 = 2;
const UI_MENU_ITEM_HEIGHT: i16 = 12;
const UI_FOOTER_HEIGHT: i16 = 12;
const UI_SCROLLBAR_WIDTH: i16 = 3;
const UI_VK_ORIGIN_X: i16 = 1;
const UI_VK_ORIGIN_Y: i16 = 29;
const UI_VK_ROW_COUNT: u8 = 3;
const UI_VK_VALIDATOR_TIMEOUT_MS: u32 = 4000;
const UI_VK_BACKSPACE_W: i16 = 16;
const UI_VK_BACKSPACE_H: i16 = 9;
const UI_VK_ENTER_W: i16 = 24;
const UI_VK_ENTER_H: i16 = 11;

const UI_VK_ENTER_KEY: u8 = b'\r';
const UI_VK_BACKSPACE_KEY: u8 = b'\x08';

const LAYOUT_VERTICAL: u8 = 0;
const LAYOUT_HORIZONTAL: u8 = 1;

#[derive(Copy, Clone)]
struct UiLayoutStack {
    x: i16,
    y: i16,
    width: i16,
    direction: u8,
    spacing: i16,
    cursor: i16,
    centered: bool,
}

impl UiLayoutStack {
    const fn empty() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            direction: LAYOUT_VERTICAL,
            spacing: 0,
            cursor: 0,
            centered: false,
        }
    }
}

#[derive(Copy, Clone)]
struct DeferredButton {
    x: i16,
    y: i16,
    width: i16,
    height: i16,
    text_offset: usize,
    text_len: usize,
    focused: bool,
}

impl DeferredButton {
    const fn empty() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            text_offset: 0,
            text_len: 0,
            focused: false,
        }
    }
}

#[derive(Copy, Clone)]
struct UiContext {
    layout_stack: [UiLayoutStack; UI_MAX_LAYOUT_DEPTH],
    layout_depth: i8,
    focus_index: i16,
    focus_count: i16,
    prev_focus_count: i16,
    last_key: u8,
    last_type: u8,
    has_input: bool,
    ok_pressed: bool,
    back_pressed: bool,
    menu_active: bool,
    menu_scroll_ptr: usize,
    menu_scroll: i16,
    menu_visible: i16,
    menu_total: i16,
    menu_y_start: i16,
    menu_first_item_focus: i16,
    use_absolute: bool,
    abs_x: i16,
    abs_y: i16,
    deferred_buttons: [DeferredButton; UI_MAX_DEFERRED],
    deferred_count: i8,
    deferred_text: [u8; UI_DEFERRED_TEXT_CAP],
    deferred_text_len: usize,
}

impl UiContext {
    const fn new() -> Self {
        Self {
            layout_stack: [UiLayoutStack::empty(); UI_MAX_LAYOUT_DEPTH],
            layout_depth: -1,
            focus_index: 0,
            focus_count: 0,
            prev_focus_count: 0,
            last_key: 0,
            last_type: 0,
            has_input: false,
            ok_pressed: false,
            back_pressed: false,
            menu_active: false,
            menu_scroll_ptr: 0,
            menu_scroll: 0,
            menu_visible: 0,
            menu_total: 0,
            menu_y_start: 0,
            menu_first_item_focus: 0,
            use_absolute: false,
            abs_x: 0,
            abs_y: 0,
            deferred_buttons: [DeferredButton::empty(); UI_MAX_DEFERRED],
            deferred_count: 0,
            deferred_text: [0u8; UI_DEFERRED_TEXT_CAP],
            deferred_text_len: 0,
        }
    }
}

static UI_CTX: crate::AppCell<UiContext> = crate::AppCell::new(UiContext::new());

fn with_ctx<F, R>(func: F) -> R
where
    F: FnOnce(&mut UiContext) -> R,
{
    let mut ctx = UI_CTX.get();
    let result = func(&mut ctx);
    UI_CTX.set(ctx);
    result
}

fn current_layout(ctx: &UiContext) -> Option<UiLayoutStack> {
    if ctx.layout_depth < 0 {
        None
    } else {
        Some(ctx.layout_stack[ctx.layout_depth as usize])
    }
}

fn set_current_layout(ctx: &mut UiContext, layout: UiLayoutStack) {
    if ctx.layout_depth < 0 {
        return;
    }
    ctx.layout_stack[ctx.layout_depth as usize] = layout;
}

fn font_height(font_id: u32) -> i16 {
    match font_id {
        font::PRIMARY => UI_FONT_HEIGHT_PRIMARY,
        _ => UI_FONT_HEIGHT_SECONDARY,
    }
}

fn layout_next(ctx: &mut UiContext, width: i16, height: i16) -> (i16, i16, i16) {
    if ctx.use_absolute {
        ctx.use_absolute = false;
        return (ctx.abs_x, ctx.abs_y, width);
    }

    let mut layout = match current_layout(ctx) {
        Some(layout) => layout,
        None => {
            return (0, 0, UI_SCREEN_WIDTH);
        }
    };

    let (x, y, w) = if layout.direction == LAYOUT_VERTICAL {
        let x = layout.x;
        let y = layout.y + layout.cursor;
        let w = layout.width;
        layout.cursor = layout.cursor.saturating_add(height + layout.spacing);
        (x, y, w)
    } else {
        let x = layout.x + layout.cursor;
        let y = layout.y;
        let w = width;
        layout.cursor = layout.cursor.saturating_add(width + layout.spacing);
        (x, y, w)
    };

    set_current_layout(ctx, layout);
    (x, y, w)
}

fn register_focusable(ctx: &mut UiContext) -> i16 {
    if ctx.focus_count >= UI_MAX_FOCUSABLE {
        return -1;
    }
    let idx = ctx.focus_count;
    ctx.focus_count += 1;
    idx
}

fn is_focused(ctx: &UiContext, idx: i16) -> bool {
    ctx.focus_index == idx
}

fn is_activated(ctx: &UiContext, idx: i16) -> bool {
    is_focused(ctx, idx) && ctx.ok_pressed
}

fn in_centered_hstack(ctx: &UiContext) -> bool {
    if let Some(layout) = current_layout(ctx) {
        layout.direction == LAYOUT_HORIZONTAL && layout.centered
    } else {
        false
    }
}

fn draw_button_internal(x: i16, y: i16, w: i16, h: i16, text: &str, focused: bool) {
    canvas_set_font(font::SECONDARY);
    if focused {
        canvas_set_color(color::BLACK);
        canvas_draw_rbox(x as i32, y as i32, w as u32, h as u32, 2);
        canvas_set_color(color::WHITE);
        canvas_draw_str(x as i32 + UI_BUTTON_PADDING_X as i32, y as i32 + h as i32 - UI_BUTTON_PADDING_Y as i32, text);
    } else {
        canvas_set_color(color::BLACK);
        canvas_draw_rframe(x as i32, y as i32, w as u32, h as u32, 2);
        canvas_draw_str(x as i32 + UI_BUTTON_PADDING_X as i32, y as i32 + h as i32 - UI_BUTTON_PADDING_Y as i32, text);
    }
}

fn defer_button(ctx: &mut UiContext, x: i16, y: i16, w: i16, h: i16, text: &str, focused: bool) {
    if ctx.deferred_count as usize >= UI_MAX_DEFERRED {
        return;
    }

    let bytes = text.as_bytes();
    let remaining = UI_DEFERRED_TEXT_CAP.saturating_sub(ctx.deferred_text_len);
    if remaining == 0 {
        return;
    }
    let copy_len = bytes.len().min(remaining);
    let offset = ctx.deferred_text_len;
    ctx.deferred_text[offset..offset + copy_len].copy_from_slice(&bytes[..copy_len]);
    ctx.deferred_text_len += copy_len;

    ctx.deferred_buttons[ctx.deferred_count as usize] = DeferredButton {
        x,
        y,
        width: w,
        height: h,
        text_offset: offset,
        text_len: copy_len,
        focused,
    };
    ctx.deferred_count += 1;
}

pub fn ui_begin() {
    with_ctx(|ctx| {
        canvas_clear();
        ctx.prev_focus_count = ctx.focus_count;
        ctx.focus_count = 0;

        ctx.layout_depth = 0;
        ctx.layout_stack[0] = UiLayoutStack {
            x: 0,
            y: 0,
            width: UI_SCREEN_WIDTH,
            direction: LAYOUT_VERTICAL,
            spacing: 0,
            cursor: 0,
            centered: false,
        };

        ctx.menu_active = false;
        ctx.menu_scroll_ptr = 0;
        ctx.menu_scroll = 0;

        ctx.use_absolute = false;
        ctx.deferred_count = 0;
        ctx.deferred_text_len = 0;
    });
}

pub fn ui_end() {
    with_ctx(|ctx| {
        if ctx.focus_count > 0 {
            if ctx.focus_index < 0 {
                ctx.focus_index = 0;
            } else if ctx.focus_index >= ctx.focus_count {
                ctx.focus_index = ctx.focus_count - 1;
            }
        } else {
            ctx.focus_index = -1;
        }

        ctx.has_input = false;
        ctx.ok_pressed = false;
        ctx.back_pressed = false;
    });
}

pub fn ui_input(key: u8, input_type: u8) {
    with_ctx(|ctx| {
        if input_type != input::TYPE_RELEASE as u8 {
            ctx.last_key = key;
            ctx.last_type = input_type;
            ctx.has_input = true;
        }

        if input_type == input::TYPE_SHORT_PRESS as u8 || input_type == input::TYPE_REPEAT as u8 {
            match key {
                k if k == input::KEY_UP as u8 => {
                    if ctx.prev_focus_count > 0 {
                        ctx.focus_index -= 1;
                        if ctx.focus_index < 0 {
                            ctx.focus_index = ctx.prev_focus_count - 1;
                        }
                    }
                }
                k if k == input::KEY_DOWN as u8 => {
                    if ctx.prev_focus_count > 0 {
                        ctx.focus_index += 1;
                        if ctx.focus_index >= ctx.prev_focus_count {
                            ctx.focus_index = 0;
                        }
                    }
                }
                k if k == input::KEY_OK as u8 => {
                    ctx.ok_pressed = true;
                }
                k if k == input::KEY_BACK as u8 => {
                    ctx.back_pressed = true;
                }
                _ => {}
            }
        }
    });
}

pub fn ui_back_pressed() -> bool {
    with_ctx(|ctx| ctx.back_pressed)
}

pub fn ui_vstack(spacing: i16) {
    with_ctx(|ctx| {
        if ctx.layout_depth >= (UI_MAX_LAYOUT_DEPTH as i8 - 1) {
            return;
        }
        let parent = current_layout(ctx);
        let (new_x, new_y, new_width) = if let Some(parent) = parent {
            (parent.x, parent.y + parent.cursor, parent.width)
        } else {
            (0, 0, UI_SCREEN_WIDTH)
        };

        ctx.layout_depth += 1;
        ctx.layout_stack[ctx.layout_depth as usize] = UiLayoutStack {
            x: new_x,
            y: new_y,
            width: new_width,
            direction: LAYOUT_VERTICAL,
            spacing,
            cursor: 0,
            centered: false,
        };
    });
}

pub fn ui_hstack(spacing: i16) {
    with_ctx(|ctx| {
        if ctx.layout_depth >= (UI_MAX_LAYOUT_DEPTH as i8 - 1) {
            return;
        }
        let parent = current_layout(ctx);
        let (new_x, new_y, new_width) = if let Some(parent) = parent {
            (parent.x, parent.y + parent.cursor, parent.width)
        } else {
            (0, 0, UI_SCREEN_WIDTH)
        };

        ctx.layout_depth += 1;
        ctx.layout_stack[ctx.layout_depth as usize] = UiLayoutStack {
            x: new_x,
            y: new_y,
            width: new_width,
            direction: LAYOUT_HORIZONTAL,
            spacing,
            cursor: 0,
            centered: false,
        };
    });
}

pub fn ui_hstack_centered(spacing: i16) {
    with_ctx(|ctx| {
        if ctx.layout_depth >= (UI_MAX_LAYOUT_DEPTH as i8 - 1) {
            return;
        }
        let parent = current_layout(ctx);
        let (new_x, new_y, new_width) = if let Some(parent) = parent {
            (parent.x, parent.y + parent.cursor, parent.width)
        } else {
            (0, 0, UI_SCREEN_WIDTH)
        };

        ctx.deferred_count = 0;
        ctx.deferred_text_len = 0;

        ctx.layout_depth += 1;
        ctx.layout_stack[ctx.layout_depth as usize] = UiLayoutStack {
            x: new_x,
            y: new_y,
            width: new_width,
            direction: LAYOUT_HORIZONTAL,
            spacing,
            cursor: 0,
            centered: true,
        };
    });
}

pub fn ui_end_stack() {
    with_ctx(|ctx| {
        if ctx.layout_depth <= 0 {
            return;
        }

        let ending = ctx.layout_stack[ctx.layout_depth as usize];

        if ending.centered && ending.direction == LAYOUT_HORIZONTAL {
            let mut content_width = ending.cursor;
            if ending.spacing > 0 && content_width > 0 {
                content_width -= ending.spacing;
            }
            let offset = (ending.width - content_width) / 2;

            for idx in 0..ctx.deferred_count as usize {
                let btn = ctx.deferred_buttons[idx];
                let start = btn.text_offset;
                let end = start + btn.text_len;
                let text = core::str::from_utf8(&ctx.deferred_text[start..end]).unwrap_or("");
                draw_button_internal(
                    btn.x + offset,
                    btn.y,
                    btn.width,
                    btn.height,
                    text,
                    btn.focused,
                );
            }

            ctx.deferred_count = 0;
            ctx.deferred_text_len = 0;
        }

        let mut used_size = ending.cursor;
        if ending.spacing > 0 && used_size > 0 {
            used_size -= ending.spacing;
        }

        let used_height = if ending.direction == LAYOUT_HORIZONTAL {
            UI_FONT_HEIGHT_SECONDARY + UI_BUTTON_PADDING_Y * 2
        } else {
            used_size
        };

        ctx.layout_depth -= 1;
        if let Some(mut parent) = current_layout(ctx) {
            if parent.direction == LAYOUT_VERTICAL {
                parent.cursor = parent.cursor.saturating_add(used_height + parent.spacing);
                set_current_layout(ctx, parent);
            }
        }
    });
}

pub fn ui_spacer(pixels: i16) {
    with_ctx(|ctx| {
        if let Some(mut layout) = current_layout(ctx) {
            layout.cursor = layout.cursor.saturating_add(pixels);
            set_current_layout(ctx, layout);
        }
    });
}

pub fn ui_set_position(x: i16, y: i16) {
    with_ctx(|ctx| {
        ctx.use_absolute = true;
        ctx.abs_x = x;
        ctx.abs_y = y;
    });
}

pub fn ui_label(text: &str, font_id: u32, align_mode: u32) {
    with_ctx(|ctx| {
        let font_height = font_height(font_id);
        let (x, y, w) = layout_next(ctx, 0, font_height);

        canvas_set_font(font_id);
        canvas_set_color(color::BLACK);

        let text_width = canvas_string_width(text) as i16;
        let text_x = match align_mode {
            align::CENTER => x + (w - text_width) / 2,
            align::RIGHT => x + w - text_width,
            _ => x,
        };

        canvas_draw_str(text_x as i32, (y + font_height) as i32, text);
    });
}

pub fn ui_separator() {
    with_ctx(|ctx| {
        let (x, y, w) = layout_next(ctx, 0, 5);
        canvas_set_color(color::BLACK);
        canvas_draw_line(x as i32, (y + 2) as i32, (x + w - 1) as i32, (y + 2) as i32);
    });
}

pub fn ui_button(text: &str) -> bool {
    with_ctx(|ctx| {
        canvas_set_font(font::SECONDARY);
        let text_width = canvas_string_width(text) as i16;

        let btn_width = text_width + UI_BUTTON_PADDING_X * 2;
        let btn_height = UI_FONT_HEIGHT_SECONDARY + UI_BUTTON_PADDING_Y * 2;

        let (x, y, w) = layout_next(ctx, btn_width, btn_height);

        let idx = register_focusable(ctx);
        let focused = is_focused(ctx, idx);
        let activated = is_activated(ctx, idx);

        if in_centered_hstack(ctx) {
            defer_button(ctx, x, y, btn_width, btn_height, text, focused);
        } else {
            let btn_x = x + (w - btn_width) / 2;
            draw_button_internal(btn_x, y, btn_width, btn_height, text, focused);
        }

        if activated {
            crate::request_render();
        }

        activated
    })
}

pub fn ui_button_at(x: i16, y: i16, text: &str) -> bool {
    ui_set_position(x, y);
    ui_button(text)
}

pub fn ui_progress(mut value: f32, width: i16) {
    with_ctx(|ctx| {
        if value < 0.0 {
            value = 0.0;
        }
        if value > 1.0 {
            value = 1.0;
        }

        let bar_height = 8;
        let (x, y, w) = layout_next(ctx, width, bar_height);

        let bar_width = if width > 0 { width } else { w - 8 };
        let bar_x = x + (w - bar_width) / 2;

        canvas_set_color(color::BLACK);
        canvas_draw_frame(bar_x as i32, y as i32, bar_width as u32, bar_height as u32);

        let fill_width = (value * (bar_width - 2) as f32) as i16;
        if fill_width > 0 {
            canvas_draw_box((bar_x + 1) as i32, (y + 1) as i32, fill_width as u32, (bar_height - 2) as u32);
        }
    });
}

pub fn ui_icon(data: &[u8], width: u8, height: u8) {
    with_ctx(|ctx| {
        let (x, y, w) = layout_next(ctx, width as i16, height as i16);
        let icon_x = x + (w - width as i16) / 2;

        canvas_set_color(color::BLACK);
        let bytes_per_row = (width + 7) / 8;
        for iy in 0..height {
            for ix in 0..width {
                let byte_idx = (iy as usize * bytes_per_row as usize) + (ix as usize / 8);
                let bit_idx = ix % 8;
                if let Some(byte) = data.get(byte_idx) {
                    if (byte & (1 << bit_idx)) != 0 {
                        canvas_draw_dot((icon_x + ix as i16) as i32, (y + iy as i16) as i32);
                    }
                }
            }
        }
    });
}

pub fn ui_checkbox(text: &str, checked: &mut bool) -> bool {
    with_ctx(|ctx| {
        canvas_set_font(font::SECONDARY);

        let box_size = 10;
        let mut item_height = UI_FONT_HEIGHT_SECONDARY + 2;
        if item_height < box_size {
            item_height = box_size;
        }

        let (x, y, w) = layout_next(ctx, 0, item_height);

        let idx = register_focusable(ctx);
        let focused = is_focused(ctx, idx);
        let activated = is_activated(ctx, idx);

        if activated {
            *checked = !*checked;
            crate::request_render();
        }

        let box_x = x + 2;
        let box_y = y + (item_height - box_size) / 2;

        canvas_set_color(color::BLACK);
        if focused {
            canvas_draw_box(x as i32, y as i32, w as u32, item_height as u32);
            canvas_set_color(color::WHITE);
        }

        canvas_draw_frame(box_x as i32, box_y as i32, box_size as u32, box_size as u32);
        if *checked {
            canvas_draw_line((box_x + 2) as i32, (box_y + 5) as i32, (box_x + 4) as i32, (box_y + 7) as i32);
            canvas_draw_line((box_x + 4) as i32, (box_y + 7) as i32, (box_x + 7) as i32, (box_y + 2) as i32);
        }

        canvas_draw_str((box_x + box_size + 4) as i32, (y + item_height - 2) as i32, text);
        if activated {
            crate::request_render();
        }

        activated
    })
}

pub fn ui_menu_begin(scroll: &mut i16, visible: i16, total: i16) {
    with_ctx(|ctx| {
        ctx.menu_active = true;
        ctx.menu_scroll_ptr = scroll as *mut i16 as usize;
        ctx.menu_scroll = *scroll;
        ctx.menu_visible = visible;
        ctx.menu_total = total;
        ctx.menu_first_item_focus = ctx.focus_count;

        ctx.menu_y_start = if let Some(layout) = current_layout(ctx) {
            layout.y + layout.cursor
        } else {
            0
        };

        // Ensure focused item is visible before drawing any menu rows.
        if ctx.focus_index >= ctx.menu_first_item_focus {
            let focused_item = ctx.focus_index - ctx.menu_first_item_focus;
            if focused_item >= 0 && focused_item < total {
                let mut next_scroll = ctx.menu_scroll;
                if focused_item < next_scroll {
                    next_scroll = focused_item;
                } else if focused_item >= next_scroll + ctx.menu_visible {
                    next_scroll = focused_item - ctx.menu_visible + 1;
                }

                if total > ctx.menu_visible {
                    let max_scroll = total - ctx.menu_visible;
                    if next_scroll > max_scroll {
                        next_scroll = max_scroll;
                    }
                } else {
                    next_scroll = 0;
                }

                if next_scroll < 0 {
                    next_scroll = 0;
                }

                ctx.menu_scroll = next_scroll;
            }
        }
    });
}

fn menu_scroll(ctx: &mut UiContext) -> i16 {
    ctx.menu_scroll
}

fn set_menu_scroll(ctx: &mut UiContext, value: i16) {
    ctx.menu_scroll = value;
}

pub fn ui_menu_item(text: &str, index: i16) -> bool {
    with_ctx(|ctx| {
        if !ctx.menu_active || ctx.menu_scroll_ptr == 0 {
            return false;
        }

        let idx = register_focusable(ctx);
        let focused = is_focused(ctx, idx);
        let activated = is_activated(ctx, idx);

        if focused {
            let mut scroll = menu_scroll(ctx);
            if index < scroll {
                scroll = index;
            } else if index >= scroll + ctx.menu_visible {
                scroll = index - ctx.menu_visible + 1;
            }
            set_menu_scroll(ctx, scroll);
        }

        let scroll = menu_scroll(ctx);
        if index < scroll || index >= scroll + ctx.menu_visible {
            return false;
        }

        canvas_set_font(font::SECONDARY);
        let visible_index = index - scroll;
        let y = ctx.menu_y_start + visible_index * UI_MENU_ITEM_HEIGHT;
        let item_width = UI_SCREEN_WIDTH - UI_SCROLLBAR_WIDTH - 2;

        canvas_set_color(color::BLACK);
        if focused {
            canvas_draw_box(0, y as i32, item_width as u32, UI_MENU_ITEM_HEIGHT as u32);
            canvas_set_color(color::WHITE);
        }

        canvas_draw_str(2, (y + UI_MENU_ITEM_HEIGHT - 2) as i32, text);
        if activated {
            crate::request_render();
        }

        activated
    })
}

pub fn ui_menu_item_value(label: &str, value: &str, index: i16) -> bool {
    with_ctx(|ctx| {
        if !ctx.menu_active || ctx.menu_scroll_ptr == 0 {
            return false;
        }

        let idx = register_focusable(ctx);
        let focused = is_focused(ctx, idx);
        let activated = is_activated(ctx, idx);

        if focused {
            let mut scroll = menu_scroll(ctx);
            if index < scroll {
                scroll = index;
            } else if index >= scroll + ctx.menu_visible {
                scroll = index - ctx.menu_visible + 1;
            }
            set_menu_scroll(ctx, scroll);
        }

        let scroll = menu_scroll(ctx);
        if index < scroll || index >= scroll + ctx.menu_visible {
            return false;
        }

        canvas_set_font(font::SECONDARY);
        let visible_index = index - scroll;
        let y = ctx.menu_y_start + visible_index * UI_MENU_ITEM_HEIGHT;
        let item_width = UI_SCREEN_WIDTH - UI_SCROLLBAR_WIDTH - 2;

        canvas_set_color(color::BLACK);
        if focused {
            canvas_draw_box(0, y as i32, item_width as u32, UI_MENU_ITEM_HEIGHT as u32);
            canvas_set_color(color::WHITE);
        }

        canvas_draw_str(2, (y + UI_MENU_ITEM_HEIGHT - 2) as i32, label);
        let value_width = canvas_string_width(value) as i16;
        canvas_draw_str((item_width - value_width - 2) as i32, (y + UI_MENU_ITEM_HEIGHT - 2) as i32, value);
        if activated {
            crate::request_render();
        }

        activated
    })
}

pub fn ui_menu_end() {
    with_ctx(|ctx| {
        if !ctx.menu_active {
            return;
        }

        if ctx.menu_total > ctx.menu_visible {
            let scroll = ctx.menu_scroll;
            let scrollbar_height = ctx.menu_visible * UI_MENU_ITEM_HEIGHT;
            let scrollbar_x = UI_SCREEN_WIDTH - 2;

            let mut thumb_height = (scrollbar_height * ctx.menu_visible) / ctx.menu_total;
            if thumb_height < 4 {
                thumb_height = 4;
            }

            let thumb_y = ctx.menu_y_start
                + ((scrollbar_height - thumb_height) * scroll)
                    / (ctx.menu_total - ctx.menu_visible);

            canvas_set_color(color::BLACK);
            let mut y = ctx.menu_y_start;
            let end = ctx.menu_y_start + scrollbar_height;
            while y < end {
                canvas_draw_dot(scrollbar_x as i32, y as i32);
                y += 2;
            }

            canvas_draw_box((scrollbar_x - 1) as i32, thumb_y as i32, 3, thumb_height as u32);
        }

        if let Some(mut layout) = current_layout(ctx) {
            let mut visible_count = ctx.menu_visible;
            if ctx.menu_total < visible_count {
                visible_count = ctx.menu_total;
            }
            layout.cursor = layout.cursor.saturating_add(visible_count * UI_MENU_ITEM_HEIGHT + layout.spacing);
            set_current_layout(ctx, layout);
        }

        if ctx.menu_scroll_ptr != 0 {
            unsafe {
                let ptr = ctx.menu_scroll_ptr as *mut i16;
                *ptr = ctx.menu_scroll;
            }
        }

        ctx.menu_active = false;
    });
}

pub fn ui_footer_left(text: &str) -> bool {
    with_ctx(|ctx| {
        canvas_set_font(font::SECONDARY);
        let y = UI_SCREEN_HEIGHT - UI_FOOTER_HEIGHT;

        canvas_set_color(color::BLACK);
        canvas_draw_line(2, (y + 5) as i32, 6, (y + 2) as i32);
        canvas_draw_line(2, (y + 5) as i32, 6, (y + 8) as i32);
        canvas_draw_str(9, (y + UI_FOOTER_HEIGHT - 2) as i32, text);

        let activated = ctx.has_input
            && ctx.last_key == input::KEY_LEFT as u8
            && (ctx.last_type == input::TYPE_SHORT_PRESS as u8
                || ctx.last_type == input::TYPE_PRESS as u8);
        if activated {
            crate::request_render();
        }
        activated
    })
}

pub fn ui_footer_center(text: &str) -> bool {
    with_ctx(|_ctx| {
        canvas_set_font(font::SECONDARY);
        let y = UI_SCREEN_HEIGHT - UI_FOOTER_HEIGHT;

        let text_width = canvas_string_width(text) as i16;
        let total_width = text_width + 12;
        let x = (UI_SCREEN_WIDTH - total_width) / 2;

        canvas_set_color(color::BLACK);
        canvas_draw_disc((x + 4) as i32, (y + 5) as i32, 3);
        canvas_draw_str((x + 12) as i32, (y + UI_FOOTER_HEIGHT - 2) as i32, text);

        false
    })
}

pub fn ui_footer_right(text: &str) -> bool {
    with_ctx(|ctx| {
        canvas_set_font(font::SECONDARY);
        let y = UI_SCREEN_HEIGHT - UI_FOOTER_HEIGHT;

        let text_width = canvas_string_width(text) as i16;
        let x = UI_SCREEN_WIDTH - text_width - 10;

        canvas_set_color(color::BLACK);
        canvas_draw_str(x as i32, (y + UI_FOOTER_HEIGHT - 2) as i32, text);

        let arrow_x = UI_SCREEN_WIDTH - 7;
        canvas_draw_line((arrow_x + 4) as i32, (y + 5) as i32, arrow_x as i32, (y + 2) as i32);
        canvas_draw_line((arrow_x + 4) as i32, (y + 5) as i32, arrow_x as i32, (y + 8) as i32);

        let activated = ctx.has_input
            && ctx.last_key == input::KEY_RIGHT as u8
            && (ctx.last_type == input::TYPE_SHORT_PRESS as u8
                || ctx.last_type == input::TYPE_PRESS as u8);
        if activated {
            crate::request_render();
        }
        activated
    })
}

pub fn ui_get_focus() -> i16 {
    with_ctx(|ctx| ctx.focus_index)
}

pub fn ui_set_focus(index: i16) {
    with_ctx(|ctx| {
        ctx.focus_index = index;
    });
}

pub fn ui_is_focused(index: i16) -> bool {
    with_ctx(|ctx| ctx.focus_index == index)
}

#[derive(Copy, Clone)]
pub struct UiVirtualKeyboard<const N: usize> {
    buffer: [u8; N],
    pub min_len: usize,
    pub row: u8,
    pub col: u8,
    pub clear_default_text: bool,
    validator_visible: bool,
    validator_deadline_ms: u32,
    validator: Option<VirtualKeyboardValidator>,
    validator_context: usize,
    validator_message: [u8; 64],
}

impl<const N: usize> UiVirtualKeyboard<N> {
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; N],
            min_len: 1,
            row: 0,
            col: 0,
            clear_default_text: false,
            validator_visible: false,
            validator_deadline_ms: 0,
            validator: None,
            validator_context: 0,
            validator_message: [0u8; 64],
        }
    }

    pub fn text(&self) -> &str {
        let len = buffer_len(self);
        core::str::from_utf8(&self.buffer[..len]).unwrap_or("")
    }

    pub fn set_text(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let len = bytes.len().min(self.buffer.len().saturating_sub(1));
        self.buffer[..len].copy_from_slice(&bytes[..len]);
        if len < self.buffer.len() {
            self.buffer[len] = 0;
        }
    }
}

pub type VirtualKeyboardValidator = fn(text: &str, message: &mut [u8], context: usize) -> bool;

fn vk_row_size(row: u8) -> u8 {
    match row {
        0 => VK_ROW_1.len() as u8,
        1 => VK_ROW_2.len() as u8,
        2 => VK_ROW_3.len() as u8,
        _ => 0,
    }
}

fn vk_row(row: u8) -> &'static [VkKey] {
    match row {
        0 => &VK_ROW_1,
        1 => &VK_ROW_2,
        _ => &VK_ROW_3,
    }
}

fn vk_selected_char<const N: usize>(keyboard: &UiVirtualKeyboard<N>) -> u8 {
    let row = vk_row(keyboard.row);
    if row.is_empty() {
        return 0;
    }
    row[keyboard.col as usize].text
}

fn vk_is_lowercase(letter: u8) -> bool {
    letter >= b'a' && letter <= b'z'
}

fn vk_to_uppercase(letter: u8) -> u8 {
    if letter == b'_' {
        return b' ';
    }
    if vk_is_lowercase(letter) {
        return letter - 0x20;
    }
    letter
}

fn buffer_len<const N: usize>(keyboard: &UiVirtualKeyboard<N>) -> usize {
    keyboard
        .buffer
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(keyboard.buffer.len())
}

fn vk_backspace<const N: usize>(keyboard: &mut UiVirtualKeyboard<N>) {
    if keyboard.buffer.is_empty() {
        return;
    }

    if keyboard.clear_default_text {
        keyboard.buffer[0] = 0;
        keyboard.clear_default_text = false;
        return;
    }

    let len = buffer_len(keyboard);
    if len > 0 {
        keyboard.buffer[len - 1] = 0;
    }
}

fn vk_set_validator_message<const N: usize>(
    keyboard: &mut UiVirtualKeyboard<N>,
    message: Option<&str>,
) {
    if let Some(message) = message {
        let bytes = message.as_bytes();
        let len = bytes.len().min(keyboard.validator_message.len() - 1);
        keyboard.validator_message[..len].copy_from_slice(&bytes[..len]);
        keyboard.validator_message[len] = 0;
    } else {
        keyboard.validator_message[0] = 0;
    }
}

fn vk_show_validator<const N: usize>(
    keyboard: &mut UiVirtualKeyboard<N>,
    now_ms: u32,
    fallback: &str,
) {
    keyboard.validator_visible = true;
    keyboard.validator_deadline_ms = now_ms + UI_VK_VALIDATOR_TIMEOUT_MS;
    if keyboard.validator_message[0] == 0 {
        vk_set_validator_message(keyboard, Some(fallback));
    }
}

fn vk_handle_ok<const N: usize>(
    keyboard: &mut UiVirtualKeyboard<N>,
    shift: bool,
    now_ms: u32,
) -> bool {
    if keyboard.buffer.is_empty() {
        return false;
    }

    let mut selected = vk_selected_char(keyboard);
    let mut text_length = buffer_len(keyboard);

    let mut toggle_case = text_length == 0 || keyboard.clear_default_text;
    if shift {
        toggle_case = !toggle_case;
    }
    if toggle_case {
        selected = vk_to_uppercase(selected);
    }

    if selected == UI_VK_ENTER_KEY {
        if let Some(validator) = keyboard.validator {
            keyboard.validator_message[0] = 0;
            let text = core::str::from_utf8(&keyboard.buffer[..text_length]).unwrap_or("");
            let ok = validator(text, &mut keyboard.validator_message, keyboard.validator_context);
            if !ok {
                vk_show_validator(keyboard, now_ms, "Invalid input");
                return false;
            }
        }

        return text_length >= keyboard.min_len;
    }

    if selected == UI_VK_BACKSPACE_KEY {
        vk_backspace(keyboard);
        keyboard.clear_default_text = false;
        return false;
    }

    if keyboard.clear_default_text {
        text_length = 0;
    }

    if text_length + 1 < keyboard.buffer.len() {
        keyboard.buffer[text_length] = selected;
        keyboard.buffer[text_length + 1] = 0;
    }

    keyboard.clear_default_text = false;
    false
}

fn vk_move_left<const N: usize>(keyboard: &mut UiVirtualKeyboard<N>) {
    let row_len = vk_row_size(keyboard.row);
    if row_len == 0 {
        return;
    }
    if keyboard.col > 0 {
        keyboard.col -= 1;
    } else {
        keyboard.col = row_len - 1;
    }
}

fn vk_move_right<const N: usize>(keyboard: &mut UiVirtualKeyboard<N>) {
    let row_len = vk_row_size(keyboard.row);
    if row_len == 0 {
        return;
    }
    if keyboard.col + 1 < row_len {
        keyboard.col += 1;
    } else {
        keyboard.col = 0;
    }
}

fn vk_move_up<const N: usize>(keyboard: &mut UiVirtualKeyboard<N>) {
    if keyboard.row == 0 {
        return;
    }
    keyboard.row -= 1;

    let row_len = vk_row_size(keyboard.row);
    if row_len == 0 {
        keyboard.col = 0;
        return;
    }

    if keyboard.col > row_len.saturating_sub(6) {
        keyboard.col = keyboard.col.saturating_add(1);
    }
    if keyboard.col >= row_len {
        keyboard.col = row_len - 1;
    }
}

fn vk_move_down<const N: usize>(keyboard: &mut UiVirtualKeyboard<N>) {
    if keyboard.row + 1 >= UI_VK_ROW_COUNT {
        return;
    }
    keyboard.row += 1;

    let row_len = vk_row_size(keyboard.row);
    if row_len == 0 {
        keyboard.col = 0;
        return;
    }

    if keyboard.col > row_len.saturating_sub(4) && keyboard.col > 0 {
        keyboard.col -= 1;
    }
    if keyboard.col >= row_len {
        keyboard.col = row_len - 1;
    }
}

#[derive(Copy, Clone)]
struct VkKey {
    text: u8,
    x: u8,
    y: u8,
}

const VK_ROW_1: [VkKey; 14] = [
    VkKey { text: b'q', x: 1, y: 8 },
    VkKey { text: b'w', x: 10, y: 8 },
    VkKey { text: b'e', x: 19, y: 8 },
    VkKey { text: b'r', x: 28, y: 8 },
    VkKey { text: b't', x: 37, y: 8 },
    VkKey { text: b'y', x: 46, y: 8 },
    VkKey { text: b'u', x: 55, y: 8 },
    VkKey { text: b'i', x: 64, y: 8 },
    VkKey { text: b'o', x: 73, y: 8 },
    VkKey { text: b'p', x: 82, y: 8 },
    VkKey { text: b'0', x: 91, y: 8 },
    VkKey { text: b'1', x: 100, y: 8 },
    VkKey { text: b'2', x: 110, y: 8 },
    VkKey { text: b'3', x: 120, y: 8 },
];

const VK_ROW_2: [VkKey; 13] = [
    VkKey { text: b'a', x: 1, y: 20 },
    VkKey { text: b's', x: 10, y: 20 },
    VkKey { text: b'd', x: 19, y: 20 },
    VkKey { text: b'f', x: 28, y: 20 },
    VkKey { text: b'g', x: 37, y: 20 },
    VkKey { text: b'h', x: 46, y: 20 },
    VkKey { text: b'j', x: 55, y: 20 },
    VkKey { text: b'k', x: 64, y: 20 },
    VkKey { text: b'l', x: 73, y: 20 },
    VkKey { text: UI_VK_BACKSPACE_KEY, x: 82, y: 12 },
    VkKey { text: b'4', x: 100, y: 20 },
    VkKey { text: b'5', x: 110, y: 20 },
    VkKey { text: b'6', x: 120, y: 20 },
];

const VK_ROW_3: [VkKey; 12] = [
    VkKey { text: b'z', x: 1, y: 32 },
    VkKey { text: b'x', x: 10, y: 32 },
    VkKey { text: b'c', x: 19, y: 32 },
    VkKey { text: b'v', x: 28, y: 32 },
    VkKey { text: b'b', x: 37, y: 32 },
    VkKey { text: b'n', x: 46, y: 32 },
    VkKey { text: b'm', x: 55, y: 32 },
    VkKey { text: b'_', x: 64, y: 32 },
    VkKey { text: UI_VK_ENTER_KEY, x: 74, y: 23 },
    VkKey { text: b'7', x: 100, y: 32 },
    VkKey { text: b'8', x: 110, y: 32 },
    VkKey { text: b'9', x: 120, y: 32 },
];

pub fn ui_virtual_keyboard_init<const N: usize>(keyboard: &mut UiVirtualKeyboard<N>, initial: &str) {
    *keyboard = UiVirtualKeyboard::new();
    keyboard.set_text(initial);
    keyboard.validator_message[0] = 0;

    if !keyboard.buffer.is_empty() && keyboard.buffer[0] != 0 {
        keyboard.row = 2;
        keyboard.col = 8;
    }
}

pub fn ui_virtual_keyboard_set_min_length<const N: usize>(
    keyboard: &mut UiVirtualKeyboard<N>,
    min_len: usize,
) {
    keyboard.min_len = min_len;
}

pub fn ui_virtual_keyboard_set_validator<const N: usize>(
    keyboard: &mut UiVirtualKeyboard<N>,
    validator: VirtualKeyboardValidator,
    context: usize,
) {
    keyboard.validator = Some(validator);
    keyboard.validator_context = context;
}

pub fn ui_virtual_keyboard<const N: usize>(
    keyboard: &mut UiVirtualKeyboard<N>,
    header: &str,
    now_ms: u32,
) -> bool {
    if keyboard.row >= UI_VK_ROW_COUNT {
        keyboard.row = 0;
    }
    let row_size = vk_row_size(keyboard.row);
    if row_size == 0 {
        keyboard.col = 0;
    } else if keyboard.col >= row_size {
        keyboard.col = row_size - 1;
    }

    if keyboard.validator_visible && now_ms >= keyboard.validator_deadline_ms {
        keyboard.validator_visible = false;
    }

    let mut submitted = false;
    with_ctx(|ctx| {
        if ctx.has_input {
            let key = ctx.last_key;
            let input_type = ctx.last_type;

            if keyboard.validator_visible
                && (input_type == input::TYPE_SHORT_PRESS as u8
                    || input_type == input::TYPE_LONG_PRESS as u8
                    || input_type == input::TYPE_REPEAT as u8)
            {
                keyboard.validator_visible = false;
            } else if input_type == input::TYPE_SHORT_PRESS as u8 {
                match key {
                    k if k == input::KEY_UP as u8 => vk_move_up(keyboard),
                    k if k == input::KEY_DOWN as u8 => vk_move_down(keyboard),
                    k if k == input::KEY_LEFT as u8 => vk_move_left(keyboard),
                    k if k == input::KEY_RIGHT as u8 => vk_move_right(keyboard),
                    k if k == input::KEY_OK as u8 => {
                        submitted = vk_handle_ok(keyboard, false, now_ms);
                    }
                    _ => {}
                }
            } else if input_type == input::TYPE_LONG_PRESS as u8 {
                match key {
                    k if k == input::KEY_OK as u8 => {
                        submitted = vk_handle_ok(keyboard, true, now_ms);
                    }
                    k if k == input::KEY_BACK as u8 => {
                        vk_backspace(keyboard);
                    }
                    _ => {}
                }
            } else if input_type == input::TYPE_REPEAT as u8 {
                match key {
                    k if k == input::KEY_UP as u8 => vk_move_up(keyboard),
                    k if k == input::KEY_DOWN as u8 => vk_move_down(keyboard),
                    k if k == input::KEY_LEFT as u8 => vk_move_left(keyboard),
                    k if k == input::KEY_RIGHT as u8 => vk_move_right(keyboard),
                    k if k == input::KEY_BACK as u8 => vk_backspace(keyboard),
                    _ => {}
                }
            }
        }
    });

    let text_length = buffer_len(keyboard);
    let header_text = header;
    let text = core::str::from_utf8(&keyboard.buffer[..text_length]).unwrap_or("");

    canvas_set_color(color::BLACK);
    canvas_set_font(font::PRIMARY);
    canvas_draw_str(2, 8, header_text);

    canvas_draw_rframe(1, 12, 126, 15, 2);

    canvas_set_font(font::SECONDARY);
    let mut needed_width = UI_SCREEN_WIDTH - 8;
    let mut start_pos: i16 = 4;

    if canvas_string_width(text) as i16 > needed_width {
        canvas_draw_str(start_pos as i32, 22, "...");
        start_pos += 6;
        needed_width -= 8;
    }

    let mut visible_text = text;
    while canvas_string_width(visible_text) as i16 > needed_width {
        if visible_text.is_empty() {
            break;
        }
        visible_text = &visible_text[1..];
    }

    let visible_width = canvas_string_width(visible_text) as i16;
    if keyboard.clear_default_text {
        canvas_draw_rbox((start_pos - 1) as i32, 14, (visible_width + 2) as u32, 10, 2);
        canvas_set_color(color::WHITE);
    } else {
        canvas_draw_str((start_pos + visible_width + 1) as i32, 22, "|");
    }

    canvas_draw_str(start_pos as i32, 22, visible_text);

    canvas_set_font(font::KEYBOARD);

    for row in 0..UI_VK_ROW_COUNT {
        let keys = vk_row(row);
        let column_count = vk_row_size(row);

        for column in 0..column_count {
            let key = keys[column as usize];
            let selected = keyboard.row == row && keyboard.col == column;

            let key_x = UI_VK_ORIGIN_X + key.x as i16;
            let key_y = UI_VK_ORIGIN_Y + key.y as i16;

            if key.text == UI_VK_ENTER_KEY {
                canvas_set_color(color::BLACK);
                if selected {
                    canvas_draw_rbox(key_x as i32, key_y as i32, UI_VK_ENTER_W as u32, UI_VK_ENTER_H as u32, 2);
                    canvas_set_color(color::WHITE);
                } else {
                    canvas_draw_rframe(key_x as i32, key_y as i32, UI_VK_ENTER_W as u32, UI_VK_ENTER_H as u32, 2);
                }
                canvas_set_font(font::SECONDARY);
                let label = "OK";
                let label_width = canvas_string_width(label) as i16;
                let label_x = key_x + (UI_VK_ENTER_W - label_width) / 2;
                canvas_draw_str(label_x as i32, (key_y + UI_VK_ENTER_H - 2) as i32, label);
                canvas_set_font(font::KEYBOARD);
                continue;
            }

            if key.text == UI_VK_BACKSPACE_KEY {
                canvas_set_color(color::BLACK);
                if selected {
                    canvas_draw_rbox(key_x as i32, key_y as i32, UI_VK_BACKSPACE_W as u32, UI_VK_BACKSPACE_H as u32, 2);
                    canvas_set_color(color::WHITE);
                } else {
                    canvas_draw_rframe(key_x as i32, key_y as i32, UI_VK_BACKSPACE_W as u32, UI_VK_BACKSPACE_H as u32, 2);
                }
                let mid_y = key_y + (UI_VK_BACKSPACE_H / 2);
                let left_x = key_x + 3;
                let right_x = key_x + UI_VK_BACKSPACE_W - 4;
                canvas_draw_line(left_x as i32, mid_y as i32, right_x as i32, mid_y as i32);
                canvas_draw_line(left_x as i32, mid_y as i32, (left_x + 3) as i32, (mid_y - 3) as i32);
                canvas_draw_line(left_x as i32, mid_y as i32, (left_x + 3) as i32, (mid_y + 3) as i32);
                continue;
            }

            if selected {
                canvas_set_color(color::BLACK);
                canvas_draw_box((key_x - 1) as i32, (key_y - 8) as i32, 7, 10);
                canvas_set_color(color::WHITE);
            } else {
                canvas_set_color(color::BLACK);
            }

            let mut glyph = key.text;
            if keyboard.clear_default_text || (text_length == 0 && vk_is_lowercase(key.text)) {
                glyph = vk_to_uppercase(glyph);
            }

            let glyph_str = [glyph, 0];
            let glyph_text = core::str::from_utf8(&glyph_str[..1]).unwrap_or("");
            canvas_draw_str(key_x as i32, key_y as i32, glyph_text);
        }
    }

    if keyboard.validator_visible {
        canvas_set_font(font::SECONDARY);
        canvas_set_color(color::WHITE);
        canvas_draw_box(8, 10, 112, 44);
        canvas_set_color(color::BLACK);
        canvas_draw_rframe(8, 8, 112, 48, 3);
        canvas_draw_rframe(9, 9, 110, 46, 2);

        let msg_len = keyboard
            .validator_message
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(keyboard.validator_message.len());
        let message = core::str::from_utf8(&keyboard.validator_message[..msg_len]).unwrap_or("");
        let msg_width = canvas_string_width(message) as i16;
        let msg_x = (UI_SCREEN_WIDTH - msg_width) / 2;
        canvas_draw_str(msg_x as i32, 34, message);
    }

    if submitted {
        crate::request_render();
    }

    submitted
}
