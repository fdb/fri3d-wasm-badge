#![no_std]
#![deny(unsafe_code)]

// Snake — vector port to the new 296x240 color screen + ui kit.
// Game logic is unchanged from the original 128x64 mono version (board
// dimensions, fruit-spawn rules, "turn only at even cells" timing). Only
// the renderer + score/game-over UI was rewritten to use screen::* and
// ui::* — keeps the game's tuning intact while landing it in the new
// design system.

use fri3d_wasm_api as api;
use api::{screen, theme, ui};

const MAX_SNAKE_LEN: usize = 253;
const BOARD_MAX_X: u8 = 30;
const BOARD_MAX_Y: u8 = 14;
const STEP_INTERVAL_MS: u32 = 250;
const SNAKE_TIMER_MS: u32 = STEP_INTERVAL_MS;

#[derive(Copy, Clone)]
struct Point {
    x: u8,
    y: u8,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
enum GameState {
    Life = 0,
    LastChance = 1,
    GameOver = 2,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
enum Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

#[derive(Copy, Clone)]
struct SnakeState {
    points: [Point; MAX_SNAKE_LEN],
    len: u16,
    current_movement: Direction,
    next_movement: Direction,
    fruit: Point,
    state: GameState,
    last_step_ms: u32,
    initialized: bool,
}

impl SnakeState {
    const fn new() -> Self {
        Self {
            points: [Point { x: 0, y: 0 }; MAX_SNAKE_LEN],
            len: 0,
            current_movement: Direction::Right,
            next_movement: Direction::Right,
            fruit: Point { x: 0, y: 0 },
            state: GameState::Life,
            last_step_ms: 0,
            initialized: false,
        }
    }
}

static STATE: api::AppCell<SnakeState> = api::AppCell::new(SnakeState::new());

fn init_game(state: &mut SnakeState) {
    let initial = [
        Point { x: 8, y: 6 },
        Point { x: 7, y: 6 },
        Point { x: 6, y: 6 },
        Point { x: 5, y: 6 },
        Point { x: 4, y: 6 },
        Point { x: 3, y: 6 },
        Point { x: 2, y: 6 },
    ];

    for (idx, point) in initial.iter().enumerate() {
        state.points[idx] = *point;
    }
    state.len = initial.len() as u16;
    state.current_movement = Direction::Right;
    state.next_movement = Direction::Right;
    state.fruit = Point { x: 18, y: 6 };
    state.state = GameState::Life;
    state.last_step_ms = api::get_time_ms();
    state.initialized = true;
    api::start_timer_ms(SNAKE_TIMER_MS);
}

fn ensure_initialized(state: &mut SnakeState) {
    if !state.initialized {
        init_game(state);
    }
}

fn get_new_fruit(state: &SnakeState) -> Point {
    let mut buffer = [0u16; 8];
    let mut empty = 8u16 * 16u16;

    for idx in 0..state.len as usize {
        let mut point = state.points[idx];
        if (point.x % 2) != 0 || (point.y % 2) != 0 {
            continue;
        }
        point.x /= 2;
        point.y /= 2;
        buffer[point.y as usize] |= 1u16 << point.x;
        empty = empty.saturating_sub(1);
    }

    if empty == 0 {
        return Point { x: 0, y: 0 };
    }

    let mut new_fruit = api::random_range(empty as u32) as u16;

    for y in 0..8 {
        let mut mask = 1u16;
        for x in 0..16 {
            if (buffer[y] & mask) == 0 {
                if new_fruit == 0 {
                    return Point {
                        x: (x * 2) as u8,
                        y: (y * 2) as u8,
                    };
                }
                new_fruit -= 1;
            }
            mask <<= 1;
        }
    }

    Point { x: 0, y: 0 }
}

fn collision_with_frame(next_step: Point) -> bool {
    next_step.x > BOARD_MAX_X || next_step.y > BOARD_MAX_Y
}

fn collision_with_tail(state: &SnakeState, next_step: Point) -> bool {
    for idx in 0..state.len as usize {
        let point = state.points[idx];
        if point.x == next_step.x && point.y == next_step.y {
            return true;
        }
    }
    false
}

fn get_turn(state: &SnakeState) -> Direction {
    let is_orthogonal =
        ((state.current_movement as u8 + state.next_movement as u8) % 2) == 1;
    if is_orthogonal {
        state.next_movement
    } else {
        state.current_movement
    }
}

fn get_next_step(state: &SnakeState) -> Point {
    let mut next_step = state.points[0];

    match state.current_movement {
        Direction::Up => {
            next_step.y = next_step.y.wrapping_sub(1);
        }
        Direction::Right => {
            next_step.x = next_step.x.wrapping_add(1);
        }
        Direction::Down => {
            next_step.y = next_step.y.wrapping_add(1);
        }
        Direction::Left => {
            next_step.x = next_step.x.wrapping_sub(1);
        }
    }

    next_step
}

fn move_snake(state: &mut SnakeState, next_step: Point) {
    let len = state.len as usize;
    for idx in (1..=len).rev() {
        state.points[idx] = state.points[idx - 1];
    }
    state.points[0] = next_step;
}

fn process_step(state: &mut SnakeState) {
    if state.state == GameState::GameOver {
        return;
    }

    let can_turn = (state.points[0].x % 2) == 0 && (state.points[0].y % 2) == 0;
    if can_turn {
        state.current_movement = get_turn(state);
    }

    let next_step = get_next_step(state);

    let crash = collision_with_frame(next_step);
    if crash {
        if state.state == GameState::Life {
            state.state = GameState::LastChance;
            return;
        }
        if state.state == GameState::LastChance {
            state.state = GameState::GameOver;
            return;
        }
    } else if state.state == GameState::LastChance {
        state.state = GameState::Life;
    }

    if collision_with_tail(state, next_step) {
        state.state = GameState::GameOver;
        return;
    }

    let eat_fruit = next_step.x == state.fruit.x && next_step.y == state.fruit.y;
    if eat_fruit {
        state.len = state.len.saturating_add(1);
        if state.len as usize >= MAX_SNAKE_LEN {
            state.state = GameState::GameOver;
            return;
        }
    }

    move_snake(state, next_step);

    if eat_fruit {
        state.fruit = get_new_fruit(state);
    }
}

fn update_state(state: &mut SnakeState) {
    let now_ms = api::get_time_ms();
    let mut elapsed = now_ms.wrapping_sub(state.last_step_ms);

    while elapsed >= STEP_INTERVAL_MS {
        process_step(state);
        if state.state == GameState::GameOver {
            api::stop_timer();
            break;
        }
        state.last_step_ms = state.last_step_ms.wrapping_add(STEP_INTERVAL_MS);
        elapsed = now_ms.wrapping_sub(state.last_step_ms);
    }
}

// Format the score as ASCII digits into `buffer`, returning bytes written.
// Used both in the status bar and the game-over modal — kept allocation-
// free so this stays no_std-friendly without relying on `format!`.
fn write_number(value: u16, buffer: &mut [u8]) -> usize {
    if buffer.is_empty() { return 0; }
    if value == 0 { buffer[0] = b'0'; return 1; }

    let mut digits = [0u8; 5];
    let mut count = 0usize;
    let mut v = value as u32;
    while v > 0 && count < digits.len() {
        digits[count] = b'0' + (v % 10) as u8;
        v /= 10;
        count += 1;
    }
    let n = count.min(buffer.len());
    for i in 0..n { buffer[i] = digits[n - 1 - i]; }
    n
}

// ---- Vector renderer -------------------------------------------------------
// Original 128x64 mono board (BOARD_MAX_X=30, BOARD_MAX_Y=14 → 31×15 cells)
// scaled up to 8 px per cell on the 296×240 color screen. The board is
// 248×120 px, centred horizontally and parked under the status bar.

const CELL: i32 = 8;
const COLS: i32 = (BOARD_MAX_X as i32) + 1;       // 31
const ROWS: i32 = (BOARD_MAX_Y as i32) + 1;       // 15
const BOARD_W: i32 = COLS * CELL;                 // 248
const BOARD_H: i32 = ROWS * CELL;                 // 120
const BOARD_X: i32 = (296 - BOARD_W) / 2;         // 24
const BOARD_Y: i32 = 14 + (240 - 14 - 14 - BOARD_H) / 2; // 70 — centred in the body area

fn render_state(state: &SnakeState) {
    screen::clear(theme::BG);

    // Status bar shows the score in the time slot — repurposes the slot
    // since the gameplay doesn't care about wall-clock.
    let score = state.len.saturating_sub(7);
    let mut score_buf = [0u8; 8];
    let score_n = write_number(score, &mut score_buf);
    let score_str = core::str::from_utf8(&score_buf[..score_n]).unwrap_or("0");
    ui::status_bar("SNAKE", score_str);

    // Subtle dot-grid background for that vector-field feel — tiny dots
    // at every cell centre give the play field structure without noise.
    for r in 0..ROWS {
        for c in 0..COLS {
            screen::pixel(
                BOARD_X + c * CELL + CELL / 2,
                BOARD_Y + r * CELL + CELL / 2,
                theme::INK_DEEP,
            );
        }
    }

    // Board frame — 1 px outside the cells, in INK so it pops against BG.
    screen::stroke_rect(BOARD_X - 2, BOARD_Y - 2, BOARD_W + 4, BOARD_H + 4, theme::INK);

    // Fruit — 2-cell-wide pink/warn square (matches the original's "fruit
    // is bigger than a body cell" affordance, but in WARN color now).
    let fx = BOARD_X + state.fruit.x as i32 * CELL;
    let fy = BOARD_Y + state.fruit.y as i32 * CELL;
    screen::fill_rect(fx + 1, fy + 1, CELL * 2 - 2, CELL * 2 - 2, theme::WARN);

    // Snake body. Head is ACCENT (cyan), body is INK (violet) with an
    // INK_DIM inner square for segment definition.
    for idx in 0..state.len as usize {
        let p = state.points[idx];
        let px = BOARD_X + p.x as i32 * CELL;
        let py = BOARD_Y + p.y as i32 * CELL;
        if idx == 0 {
            screen::fill_rect(px, py, CELL, CELL, theme::ACCENT);
            // Tiny "eye" dot toward the direction of travel
            let (dx, dy) = match state.current_movement {
                Direction::Up    => (CELL / 2, 1),
                Direction::Down  => (CELL / 2, CELL - 2),
                Direction::Left  => (1,        CELL / 2),
                Direction::Right => (CELL - 2, CELL / 2),
            };
            screen::pixel(px + dx, py + dy, theme::BG);
        } else {
            screen::fill_rect(px, py, CELL, CELL, theme::INK);
            screen::fill_rect(px + 2, py + 2, CELL - 4, CELL - 4, theme::INK_DIM);
        }
    }

    // Hint bar (below the play field).
    if state.state == GameState::GameOver {
        ui::hint_bar("OK: RESTART  B: BACK");
    } else {
        ui::hint_bar("ARROWS: TURN  B: BACK");
    }

    // Game-over modal — re-uses ui::card's "selected" treatment for the
    // accent corner brackets, so it visually reads as "the focused thing".
    if state.state == GameState::GameOver {
        const MW: i32 = 200;
        const MH: i32 = 76;
        let mx = (296 - MW) / 2;
        let my = (240 - MH) / 2;
        screen::fill_rect(mx, my, MW, MH, theme::BG_PANEL);
        ui::card(mx, my, MW, MH, /*selected=*/ true);

        ui::text_centered(mx, mx + MW, my + 22, "GAME OVER", theme::WHITE, api::font::PRIMARY);

        // "SCORE: 42" line in accent.
        let mut buf = [0u8; 24];
        let prefix = b"SCORE: ";
        buf[..prefix.len()].copy_from_slice(prefix);
        let n = write_number(score, &mut buf[prefix.len()..]);
        if let Ok(text) = core::str::from_utf8(&buf[..prefix.len() + n]) {
            ui::text_centered(mx, mx + MW, my + 42, text, theme::ACCENT, api::font::PRIMARY);
        }

        ui::text_centered(mx, mx + MW, my + 62, "PRESS A TO RESTART", theme::INK_DIM, api::font::PRIMARY);
    }
}

fn render_impl() {
    let mut state = STATE.get();
    ensure_initialized(&mut state);
    update_state(&mut state);
    render_state(&state);
    STATE.set(state);
}

fn on_input_impl(key: u32, kind: u32) {
    let mut state = STATE.get();
    ensure_initialized(&mut state);

    if key == api::input::KEY_BACK {
        if kind == api::input::TYPE_SHORT_PRESS || kind == api::input::TYPE_LONG_PRESS {
            api::exit_to_launcher();
        }
        STATE.set(state);
        return;
    }

    if kind != api::input::TYPE_PRESS {
        STATE.set(state);
        return;
    }

    if key == api::input::KEY_OK {
        if state.state == GameState::GameOver {
            init_game(&mut state);
        }
        STATE.set(state);
        return;
    }

    if state.state == GameState::GameOver {
        STATE.set(state);
        return;
    }

    match key {
        api::input::KEY_UP => state.next_movement = Direction::Up,
        api::input::KEY_DOWN => state.next_movement = Direction::Down,
        api::input::KEY_LEFT => state.next_movement = Direction::Left,
        api::input::KEY_RIGHT => state.next_movement = Direction::Right,
        _ => {}
    }

    STATE.set(state);
}

api::export_render!(render_impl);
api::export_on_input!(on_input_impl);
api::wasm_panic_handler!();
