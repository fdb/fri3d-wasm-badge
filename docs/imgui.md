# IMGUI View System

An immediate-mode GUI for Fri3d badge apps. It lives in
`fri3d_wasm_api::imgui` and draws on the 160×120 monochrome canvas through
the kernel's canvas imports. Inspired by Flipper Zero's view system, reduced
to what a `no_std`, no-allocation WASM app needs.

## Table of Contents

- [Design Philosophy](#design-philosophy)
- [Quick Start](#quick-start)
- [Frame Lifecycle](#frame-lifecycle)
- [Input Handling](#input-handling)
- [Layout System](#layout-system)
- [Widgets](#widgets)
- [Focus Navigation](#focus-navigation)
- [Menu System](#menu-system)
- [Footer Buttons](#footer-buttons)
- [Virtual Keyboard](#virtual-keyboard)
- [Complete Example](#complete-example)
- [API Reference](#api-reference)
- [Implementation Notes](#implementation-notes)

---

## Design Philosophy

### Retained Mode vs Immediate Mode

Flipper Zero uses retained mode: allocate a widget, add items with callbacks,
register it with a dispatcher, free it later.

This library uses immediate mode. Widgets exist only while `render()` runs.
A widget call draws the widget and returns `true` when the user activated it:

```rust
fn render_impl() {
    imgui::ui_begin();
    if imgui::ui_button("Option A") {
        handle_option_a();
    }
    if imgui::ui_button("Option B") {
        handle_option_b();
    }
    imgui::ui_end();
}
```

### Key Principles

1. **No widget allocation.** Widgets are function calls. Nothing is stored
   between frames except focus and layout bookkeeping.
2. **Immediate returns.** Interactive widgets return `true` when activated.
3. **State lives in the app.** The app owns its data in `static AppCell<T>`
   values. The UI only reflects it.
4. **Minimal persistence.** One fixed `UiContext` holds the focus index and
   the layout stack. No heap, no `alloc`.
5. **WASM-friendly.** No callbacks, no function pointers cross the host
   boundary. The only function pointer is the optional keyboard validator,
   which stays inside the app.

---

## Quick Start

A minimal app:

```rust
#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;
use fri3d_wasm_api::{align, font, imgui, input};

static COUNTER: api::AppCell<i32> = api::AppCell::new(0);

fn render_impl() {
    imgui::ui_begin();

    imgui::ui_label("Counter", font::PRIMARY, align::CENTER);
    imgui::ui_spacer(8);

    if imgui::ui_button("Increment") {
        COUNTER.set(COUNTER.get() + 1);
    }

    imgui::ui_end();
}

fn on_input_impl(key: u32, kind: u32) {
    imgui::ui_input(key as u8, kind as u8);
    if key == input::KEY_BACK && kind == input::TYPE_SHORT_PRESS {
        api::exit_to_launcher();
    }
}

api::export_render!(render_impl);
api::export_on_input!(on_input_impl);
api::wasm_panic_handler!();
```

`AppCell<T>` is a `Copy`-only cell that is `Sync` by construction, because a
WASM app has one thread. Read with `get()`, write back with `set()`.

The `export_*!` macros define the `render` and `on_input` symbols the
kernel looks up, so the app's own functions need other names
(`render_impl`, `on_input_impl` by convention).

---

## Frame Lifecycle

The kernel calls `render()` only when something changed: an input event, an
app timer tick, or a `request_render()`. An idle app costs nothing.

Every render follows the same shape:

```rust
fn render_impl() {
    imgui::ui_begin();   // clears the canvas, resets layout and focus count
    // ... widgets ...
    imgui::ui_end();     // clamps focus, consumes the frame's input
}
```

### What Happens Each Frame

1. `ui_begin()` **clears the whole canvas**, resets the layout stack to one
   full-width vertical stack at (0, 0), sets the focusable count to zero,
   and forgets any pending menu or deferred buttons.
2. Each widget takes the next slot from the current layout, registers itself
   as focusable if it is interactive, draws itself, and reports activation.
3. `ui_end()` clamps the focus index to `0..focus_count`, or sets it to `-1`
   when the frame had no focusable widgets, and clears the frame's input.

### `ui_begin()` clears the canvas

This matters when an app mixes IMGUI widgets with its own canvas drawing:

- Draw custom content **after** `ui_begin()`, never before it.
- When a screen needs a footer plus free-form content, call
  `ui_begin()` → footer widgets → `ui_end()` first, then draw the custom
  content with the canvas functions. The launcher's info screen does this.

```rust
// Footer first, then free-form drawing on the same frame.
imgui::ui_begin();
imgui::ui_footer_left("Back");
imgui::ui_footer_right("Open");
imgui::ui_end();

api::canvas_set_font(font::SECONDARY);
api::canvas_draw_str(2, 30, "Custom content here");
```

---

## Input Handling

Forward every input event to the UI from the app's `on_input` export:

```rust
fn on_input_impl(key: u32, kind: u32) {
    imgui::ui_input(key as u8, kind as u8);
    // app-specific handling follows
}
```

`ui_input` records the last key and type for the next frame and moves focus
on Up/Down. It does not draw. The kernel renders after every delivered input
event, so the next `render()` sees `ok_pressed` or `back_pressed` set.

### Input Types

`fri3d_wasm_api::input`:

| Constant | Meaning |
| --- | --- |
| `TYPE_PRESS` | key went down |
| `TYPE_RELEASE` | key went up |
| `TYPE_SHORT_PRESS` | released before 300 ms |
| `TYPE_LONG_PRESS` | held 300 ms, fires once |
| `TYPE_REPEAT` | every 150 ms after a long press |

The UI reacts to `TYPE_SHORT_PRESS` and `TYPE_REPEAT` for focus moves and
activation. Footer buttons also accept `TYPE_PRESS`.

### Input Keys

| Constant | Badge | UI behaviour |
| --- | --- | --- |
| `KEY_UP` / `KEY_DOWN` | joystick | move focus, wrap around |
| `KEY_LEFT` / `KEY_RIGHT` | joystick | footer buttons; free for value rows |
| `KEY_OK` | A | activate the focused widget |
| `KEY_BACK` | X | sets `ui_back_pressed()`; convention: exit or go back |
| `KEY_MENU` | MENU | kernel home key |

`KEY_MENU` belongs to the kernel. A short press returns to the launcher
before the app sees it. The app still receives the press, release and long
press, so a game can pause on `(KEY_MENU, TYPE_PRESS)`.

### Built-in Navigation

- Up/Down: previous/next focusable widget, wrapping at both ends. Focus
  moves against the *previous* frame's widget count, so the first frame
  after a screen change settles focus at `ui_end()`.
- OK: the focused widget returns `true` this frame.
- Back: `ui_back_pressed()` returns `true` this frame. The library never
  exits the app by itself.

---

## Layout System

Every widget asks the current layout for a position. The root layout is a
full-width vertical stack. Widgets advance the stack cursor by their height.

### Vertical Stack

```rust
imgui::ui_begin();                 // root vertical stack, spacing 0
imgui::ui_label("Title", font::PRIMARY, align::CENTER);
imgui::ui_separator();
imgui::ui_button("One");           // each widget lands under the previous
imgui::ui_button("Two");
imgui::ui_end();
```

### Explicit Stacks

`ui_vstack(spacing)` opens a nested vertical stack with its own spacing.
`ui_end_stack()` closes it and advances the parent by the used height.

```rust
imgui::ui_vstack(4);
imgui::ui_button("Spaced");
imgui::ui_button("By 4 px");
imgui::ui_end_stack();
```

### Horizontal Stack

`ui_hstack(spacing)` lays widgets out left to right. `ui_hstack_centered`
does the same and centres the group in the parent's width by deferring
the draw until `ui_end_stack()`.

```rust
imgui::ui_hstack_centered(4);
if imgui::ui_button("+") { COUNTER.set(COUNTER.get() + 1); }
if imgui::ui_button("-") { COUNTER.set(COUNTER.get() - 1); }
if imgui::ui_button("Reset") { COUNTER.set(0); }
imgui::ui_end_stack();
```

A horizontal stack always reports a height of one button row to its parent.

### Nested Layouts

Stacks nest up to `UI_MAX_LAYOUT_DEPTH` (8) levels. Deeper calls are
ignored, so the frame still renders.

### Layout Functions

| Function | Effect |
| --- | --- |
| `ui_vstack(spacing)` | push a vertical stack |
| `ui_hstack(spacing)` | push a horizontal stack |
| `ui_hstack_centered(spacing)` | push a centred horizontal stack |
| `ui_end_stack()` | pop the current stack |
| `ui_spacer(pixels)` | advance the cursor |
| `ui_set_position(x, y)` | place only the next widget at an absolute point |

---

## Widgets

### Labels

```rust
imgui::ui_label("Left", font::SECONDARY, align::LEFT);
imgui::ui_label("Centred", font::PRIMARY, align::CENTER);
imgui::ui_label("Right", font::SECONDARY, align::RIGHT);
```

A label takes the current layout width and one font height
(`PRIMARY` 12 px, all others 11 px). Labels are not focusable.

Build dynamic text in a fixed buffer. The SDK has no `format!`; the
`test_ui` app shows a small `write_i32` helper, and the launcher's `Num`
type is the same idea.

### Buttons

```rust
if imgui::ui_button("Save") {
    save();
}
```

A button is focusable. It draws a rounded frame, or an inverted rounded box
when focused. Width follows the text plus padding. In a plain vertical stack
the button centres itself in the layout width.

### Absolute Positioning

```rust
if imgui::ui_button_at(100, 90, "Go") { go(); }
```

`ui_button_at` is `ui_set_position` followed by `ui_button`. The position
applies to that one widget; the layout cursor does not move.

### Progress Bars

```rust
imgui::ui_progress(PROGRESS.get(), 0);   // 0 = layout width minus 8 px
imgui::ui_progress(0.75, 60);            // fixed 60 px wide, centred
```

Value is clamped to `0.0..=1.0`. The bar is 8 px tall.

### Checkboxes

```rust
let mut enabled = ENABLED.get();
if imgui::ui_checkbox("Enable sound", &mut enabled) {
    ENABLED.set(enabled);      // toggled this frame
}
```

The checkbox toggles `checked` itself on activation and returns `true` in
that frame. The focused row inverts.

### Icons

```rust
imgui::ui_icon(&ICON_BITS, 16, 16);
```

`ui_icon` centres a 1-bit bitmap in the layout width and advances by its
height. Row stride is `ceil(width / 8)` bytes; **bit 0 is the leftmost
pixel** (LSB first). This differs from `api::canvas_draw_bitmap`, which
is MSB first and is what the launcher uses for app icons.

### Separator

```rust
imgui::ui_separator();   // 5 px tall, line on the middle row
```

---

## Focus Navigation

### How Focus Works

Every interactive widget (`ui_button`, `ui_checkbox`, `ui_menu_item`,
`ui_menu_item_value`) registers itself in call order and receives the next
index, starting at 0 each frame. `ui_input` moves the index on Up/Down.
`ui_end` clamps it. Up to `UI_MAX_FOCUSABLE` (32) widgets per frame; later
widgets draw unfocused and never activate.

Call order is the tab order. Keep it stable between frames, or the focus
jumps.

### Focus State

```rust
let idx = imgui::ui_get_focus();      // -1 when nothing is focusable
imgui::ui_set_focus(2);               // jump to the third widget
if imgui::ui_is_focused(0) { /* first widget */ }
```

`ui_get_focus` is how a value row reacts to Left/Right. The settings app
adjusts brightness only when its row is focused:

```rust
fn on_input_impl(key: u32, kind: u32) {
    let step = kind == input::TYPE_SHORT_PRESS || kind == input::TYPE_REPEAT;
    match (imgui::ui_get_focus(), key) {
        (ITEM_BRIGHTNESS, input::KEY_LEFT) if step => {
            BRIGHTNESS.set(BRIGHTNESS.get().saturating_sub(10).max(10));
        }
        (ITEM_BRIGHTNESS, input::KEY_RIGHT) if step => {
            BRIGHTNESS.set((BRIGHTNESS.get() + 10).min(100));
        }
        _ => {}
    }
    imgui::ui_input(key as u8, kind as u8);
}
```

Read the focus **before** `ui_input`, or after; both work, because Left and
Right do not move focus. Do the read before `ui_input` when the same handler
also maps Up/Down.

### Visual Feedback

| Widget | Focused look |
| --- | --- |
| button | inverted rounded box |
| checkbox | inverted full row |
| menu item | inverted full row |

---

## Menu System

A scrollable list with the item height fixed at 12 px. The app owns the
scroll position and passes it in by reference each frame:

```rust
static SCROLL: api::AppCell<i16> = api::AppCell::new(0);

fn render_impl() {
    imgui::ui_begin();
    imgui::ui_label("Main Menu", font::PRIMARY, align::CENTER);
    imgui::ui_separator();

    let mut scroll = SCROLL.get();
    imgui::ui_menu_begin(&mut scroll, 6, 3);      // 6 visible rows, 3 items
    if imgui::ui_menu_item("Play", 0) { start_game(); }
    if imgui::ui_menu_item("Scores", 1) { show_scores(); }
    if imgui::ui_menu_item("Quit", 2) { api::exit_to_launcher(); }
    imgui::ui_menu_end();
    SCROLL.set(scroll);

    imgui::ui_end();
}
```

`ui_menu_begin(scroll, visible, total)` scrolls so the focused item is
visible. `ui_menu_item(text, index)` draws item `index` when it is inside the
window and returns `true` on activation. `ui_menu_end()` draws a dotted
scrollbar with a solid thumb when `total > visible`, writes the scroll
position back, and advances the layout by the visible rows.

Item `index` values must be `0..total` in call order. Items outside the
window are skipped cheaply, so calling all items every frame is fine.

With an 11 px title and a separator, 6 rows (72 px) fit above a footer on
the 120 px screen.

### Menu with Values

```rust
imgui::ui_menu_item_value("Brightness", pct.as_str(), 0);
imgui::ui_menu_item_value("Sound", if SOUND.get() { "On" } else { "Off" }, 1);
```

Label on the left, value right-aligned. Same focus and return semantics as
`ui_menu_item`.

### Adjusting Values with Left/Right

Combine `ui_menu_item_value` with `ui_get_focus` in `on_input` as shown in
[Focus State](#focus-state). The settings app is the reference
implementation: the row index constants double as focus indices because the
menu is the only focusable group on that screen.

---

## Footer Buttons

Footer hints sit in the bottom 12 px and react to Left and Right:

```rust
imgui::ui_begin();
// ... content ...
if imgui::ui_footer_left("Back") { go_back(); }        // "< Back", Left key
imgui::ui_footer_center("Select");                      // "● Select", no key
if imgui::ui_footer_right("Next") { go_next(); }       // "Next >", Right key
imgui::ui_end();
```

Footer buttons are not focusable. `ui_footer_left` returns `true` on a
Left press or short press, `ui_footer_right` on Right. `ui_footer_center`
always returns `false`; it is a hint for OK.

Footers draw at a fixed position, so call them anywhere after `ui_begin()`.

---

## Virtual Keyboard

A three-row on-screen keyboard for short text. The buffer size is a const
generic; the text is NUL-terminated inside it, so `N` bytes hold `N - 1`
characters.

```rust
static KEYBOARD: api::AppCell<imgui::UiVirtualKeyboard<32>> =
    api::AppCell::new(imgui::UiVirtualKeyboard::new());

fn init_keyboard() {
    let mut kb = KEYBOARD.get();
    imgui::ui_virtual_keyboard_init(&mut kb, "guest");   // initial text
    imgui::ui_virtual_keyboard_set_min_length(&mut kb, 3);
    kb.clear_default_text = true;   // first keystroke replaces "guest"
    KEYBOARD.set(kb);
}

fn validator(text: &str, message: &mut [u8], _context: usize) -> bool {
    if text.contains(' ') {
        let msg = b"No spaces";
        message[..msg.len()].copy_from_slice(msg);
        message[msg.len()] = 0;
        return false;
    }
    true
}

fn render_impl() {
    imgui::ui_begin();
    let now_ms = api::get_time_ms();
    let mut kb = KEYBOARD.get();
    imgui::ui_virtual_keyboard_set_validator(&mut kb, validator, 0);
    if imgui::ui_virtual_keyboard(&mut kb, "Enter Name", now_ms) {
        save_name(kb.text());
    }
    KEYBOARD.set(kb);
    imgui::ui_end();
}
```

Keys: joystick moves the cursor (repeat while held), OK types, long OK
submits, long Back or Back repeat deletes. Submit is refused below
`min_len`, and a failing validator shows its message for 4 s (hence the
`now_ms` argument). Return value is `true` on a successful submit.

`clear_default_text` shows the initial text selected; the first typed
character replaces it. `text()` returns the current string.

---

## Complete Example

A counter with a settings menu and a footer, switching between two screens.
This compiles as an app crate that depends on `fri3d-wasm-api`.

```rust
#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;
use fri3d_wasm_api::{align, font, imgui, input};

#[derive(Copy, Clone, PartialEq, Eq)]
enum Screen {
    Counter,
    Options,
}

static SCREEN: api::AppCell<Screen> = api::AppCell::new(Screen::Counter);
static COUNT: api::AppCell<i32> = api::AppCell::new(0);
static STEP: api::AppCell<i32> = api::AppCell::new(1);
static SOUND: api::AppCell<bool> = api::AppCell::new(true);
static SCROLL: api::AppCell<i16> = api::AppCell::new(0);

const ROW_STEP: i16 = 0;
const ROW_SOUND: i16 = 1;
const ROW_RESET: i16 = 2;

/// Fixed-capacity decimal formatter. No allocation.
struct Num {
    buf: [u8; 12],
    len: usize,
}

impl Num {
    fn of(mut n: i32) -> Self {
        let mut s = Num { buf: [0; 12], len: 0 };
        if n < 0 {
            s.buf[0] = b'-';
            s.len = 1;
            n = -n;
        }
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
            s.buf[s.len] = digits[i];
            s.len += 1;
        }
        s
    }
    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

fn render_counter() {
    imgui::ui_label("Counter", font::PRIMARY, align::CENTER);
    imgui::ui_spacer(10);
    imgui::ui_label(Num::of(COUNT.get()).as_str(), font::PRIMARY, align::CENTER);
    imgui::ui_spacer(10);

    imgui::ui_hstack_centered(6);
    if imgui::ui_button("-") {
        COUNT.set(COUNT.get() - STEP.get());
    }
    if imgui::ui_button("+") {
        COUNT.set(COUNT.get() + STEP.get());
    }
    imgui::ui_end_stack();

    imgui::ui_footer_left("Exit");
    if imgui::ui_footer_right("Options") {
        SCREEN.set(Screen::Options);
    }
}

fn render_options() {
    imgui::ui_label("Options", font::PRIMARY, align::CENTER);
    imgui::ui_separator();

    let mut scroll = SCROLL.get();
    imgui::ui_menu_begin(&mut scroll, 5, 3);
    imgui::ui_menu_item_value("Step", Num::of(STEP.get()).as_str(), ROW_STEP);
    if imgui::ui_menu_item_value("Sound", if SOUND.get() { "On" } else { "Off" }, ROW_SOUND) {
        SOUND.set(!SOUND.get());
    }
    if imgui::ui_menu_item("Reset counter", ROW_RESET) {
        COUNT.set(0);
    }
    imgui::ui_menu_end();
    SCROLL.set(scroll);

    if imgui::ui_footer_left("Back") {
        SCREEN.set(Screen::Counter);
    }
}

fn render_impl() {
    imgui::ui_begin();
    match SCREEN.get() {
        Screen::Counter => render_counter(),
        Screen::Options => render_options(),
    }
    imgui::ui_end();
}

fn on_input_impl(key: u32, kind: u32) {
    let short = kind == input::TYPE_SHORT_PRESS;
    let step = short || kind == input::TYPE_REPEAT;

    match SCREEN.get() {
        Screen::Counter => {
            if key == input::KEY_BACK && short {
                api::exit_to_launcher();
                return;
            }
        }
        Screen::Options => {
            if key == input::KEY_BACK && short {
                SCREEN.set(Screen::Counter);
                api::request_render();
                return;
            }
            // Left/Right adjust the focused value row.
            if imgui::ui_get_focus() == ROW_STEP && step {
                if key == input::KEY_LEFT {
                    STEP.set((STEP.get() - 1).max(1));
                } else if key == input::KEY_RIGHT {
                    STEP.set((STEP.get() + 1).min(10));
                }
            }
        }
    }

    imgui::ui_input(key as u8, kind as u8);
}

api::export_render!(render_impl);
api::export_on_input!(on_input_impl);
api::wasm_panic_handler!();
```

Note the screen switch on Back calls `api::request_render()`: the kernel
already renders after each input event, but the explicit request costs
nothing and keeps the intent visible. Widgets that activate call
`request_render()` themselves.

---

## API Reference

All functions live in `fri3d_wasm_api::imgui`. Coordinates and sizes are
`i16` pixels. Constants come from `fri3d_wasm_api::{font, align, input}`.

### Frame

| Signature | Notes |
| --- | --- |
| `fn ui_begin()` | clears the canvas, resets layout, focus count, deferred buttons |
| `fn ui_end()` | clamps focus, consumes this frame's input |
| `fn ui_input(key: u8, input_type: u8)` | call from `on_input`; Up/Down move focus, OK/Back set flags |
| `fn ui_back_pressed() -> bool` | Back short press or repeat seen this frame |

### Layout

| Signature | Notes |
| --- | --- |
| `fn ui_vstack(spacing: i16)` | push vertical stack |
| `fn ui_hstack(spacing: i16)` | push horizontal stack |
| `fn ui_hstack_centered(spacing: i16)` | push centred horizontal stack; buttons inside are drawn at `ui_end_stack` |
| `fn ui_end_stack()` | pop; advances parent by used height (one button row for horizontal) |
| `fn ui_spacer(pixels: i16)` | advance cursor |
| `fn ui_set_position(x: i16, y: i16)` | next widget only |

### Widgets

| Signature | Focusable | Returns |
| --- | --- | --- |
| `fn ui_label(text: &str, font_id: u32, align_mode: u32)` | no | — |
| `fn ui_separator()` | no | — |
| `fn ui_button(text: &str) -> bool` | yes | activated |
| `fn ui_button_at(x: i16, y: i16, text: &str) -> bool` | yes | activated |
| `fn ui_progress(value: f32, width: i16)` | no | — (`width` 0 = layout width − 8) |
| `fn ui_icon(data: &[u8], width: u8, height: u8)` | no | — (LSB-first rows) |
| `fn ui_checkbox(text: &str, checked: &mut bool) -> bool` | yes | toggled this frame |

### Focus

| Signature | Notes |
| --- | --- |
| `fn ui_get_focus() -> i16` | `-1` when nothing is focusable |
| `fn ui_set_focus(index: i16)` | takes effect immediately |
| `fn ui_is_focused(index: i16) -> bool` | |

### Menu

| Signature | Notes |
| --- | --- |
| `fn ui_menu_begin(scroll: &mut i16, visible: i16, total: i16)` | `scroll` is written back in `ui_menu_end` |
| `fn ui_menu_item(text: &str, index: i16) -> bool` | activated |
| `fn ui_menu_item_value(label: &str, value: &str, index: i16) -> bool` | activated; value right-aligned |
| `fn ui_menu_end()` | scrollbar, layout advance |

Menu rows are `UI_MENU_ITEM_HEIGHT` = 12 px; the scrollbar takes 3 px on
the right.

### Footer

| Signature | Key | Returns |
| --- | --- | --- |
| `fn ui_footer_left(text: &str) -> bool` | Left | pressed |
| `fn ui_footer_center(text: &str) -> bool` | — | always `false` |
| `fn ui_footer_right(text: &str) -> bool` | Right | pressed |

The footer occupies the bottom `UI_FOOTER_HEIGHT` = 12 px.

### Virtual keyboard

| Signature | Notes |
| --- | --- |
| `struct UiVirtualKeyboard<const N: usize>` | `Copy`; fields `min_len`, `row`, `col`, `clear_default_text` are public |
| `const fn UiVirtualKeyboard::new() -> Self` | empty, `min_len` 1 |
| `fn text(&self) -> &str` | current text |
| `fn set_text(&mut self, text: &str)` | clipped to `N - 1` bytes |
| `type VirtualKeyboardValidator = fn(text: &str, message: &mut [u8], context: usize) -> bool` | write a NUL-terminated message (≤ 63 bytes) on `false` |
| `fn ui_virtual_keyboard_init<const N: usize>(kb: &mut UiVirtualKeyboard<N>, initial: &str)` | resets state, sets text, moves the cursor to OK when text is non-empty |
| `fn ui_virtual_keyboard_set_min_length<const N: usize>(kb: &mut UiVirtualKeyboard<N>, min_len: usize)` | |
| `fn ui_virtual_keyboard_set_validator<const N: usize>(kb: &mut UiVirtualKeyboard<N>, validator: VirtualKeyboardValidator, context: usize)` | |
| `fn ui_virtual_keyboard<const N: usize>(kb: &mut UiVirtualKeyboard<N>, header: &str, now_ms: u32) -> bool` | draws and handles input; `true` on submit |

### Screen constants

The library derives its geometry from `fri3d_wasm_api::SCREEN_WIDTH`
(160) and `SCREEN_HEIGHT` (120). Font heights: primary 12 px, others 11 px.
Button padding 4 px horizontal, 2 px vertical.

---

## Implementation Notes

- **No allocation.** The whole library state is one `UiContext` value in a
  `static AppCell`. Each call copies it out, mutates it, and copies it back.
  It contains the layout stack (8 entries), focus counters, the last input,
  menu bookkeeping, and the deferred-button table.
- **Bounded tables.** `UI_MAX_LAYOUT_DEPTH` = 8 stacks,
  `UI_MAX_FOCUSABLE` = 32 focusable widgets per frame,
  `UI_MAX_DEFERRED` = 16 deferred buttons with 128 bytes of text between
  them. Past a bound the call is ignored, never a panic.
- **Deferred buttons.** Inside `ui_hstack_centered` a button cannot know
  the group width yet, so it is recorded and drawn at `ui_end_stack()`
  once the centring offset is known.
- **Menu scroll pointer.** `ui_menu_begin` keeps the address of the
  caller's `scroll` and `ui_menu_end` writes through it. Keep the `scroll`
  variable alive until `ui_menu_end()`.
- **Rendering on activation.** Every widget that returns `true`, and a
  successful keyboard submit, calls `api::request_render()` so the kernel
  draws the resulting state in the same step, never one frame late.
- **Full redraw.** The canvas is 19 200 bytes and the kernel only renders
  on change, so each frame redraws everything. No dirty rectangles.
- **Strings** are `&str`; the SDK copies them to a 256-byte stack buffer
  and NUL-terminates them for the host. Longer strings are clipped.
- **Fonts.** `font::PRIMARY` (bold 8 px Helvetica) for titles and button
  labels in the launcher, `font::SECONDARY` for body text and menu rows,
  `font::KEYBOARD` for the keyboard glyphs, `font::BIG_NUMBERS` for
  large digits.
