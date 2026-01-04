#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;

const MAX_SNAKE_LEN: usize = 253;
const BOARD_MAX_X: u8 = 30;
const BOARD_MAX_Y: u8 = 14;
const STEP_INTERVAL_MS: u32 = 250;

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
        state.last_step_ms = state.last_step_ms.wrapping_add(STEP_INTERVAL_MS);
        elapsed = now_ms.wrapping_sub(state.last_step_ms);
    }
}

fn write_score(score: u16, buffer: &mut [u8; 16]) -> usize {
    let prefix = b"Score: ";
    buffer[..prefix.len()].copy_from_slice(prefix);

    let mut digits = [0u8; 5];
    let mut count = 0usize;
    let mut value = score as u32;

    if value == 0 {
        digits[0] = b'0';
        count = 1;
    } else {
        while value > 0 {
            digits[count] = b'0' + (value % 10) as u8;
            value /= 10;
            count += 1;
        }
    }

    for idx in 0..count {
        buffer[prefix.len() + idx] = digits[count - 1 - idx];
    }

    prefix.len() + count
}

fn render_state(state: &SnakeState) {
    api::canvas_set_color(api::color::BLACK);

    api::canvas_draw_frame(0, 0, 128, 64);

    let fruit_x = state.fruit.x as i32 * 4 + 1;
    let fruit_y = state.fruit.y as i32 * 4 + 1;
    api::canvas_draw_rframe(fruit_x, fruit_y, 6, 6, 2);

    for idx in 0..state.len as usize {
        let point = state.points[idx];
        let px = point.x as i32 * 4 + 2;
        let py = point.y as i32 * 4 + 2;
        api::canvas_draw_box(px, py, 4, 4);
    }

    if state.state == GameState::GameOver {
        api::canvas_set_color(api::color::WHITE);
        api::canvas_draw_box(34, 20, 62, 24);

        api::canvas_set_color(api::color::BLACK);
        api::canvas_draw_frame(34, 20, 62, 24);

        api::canvas_set_font(api::font::PRIMARY);
        api::canvas_draw_str(37, 31, "Game Over");

        api::canvas_set_font(api::font::SECONDARY);
        let score = state.len.saturating_sub(7);
        let mut buffer = [0u8; 16];
        let len = write_score(score, &mut buffer);
        if let Ok(text) = core::str::from_utf8(&buffer[..len]) {
            let width = api::canvas_string_width(text) as i32;
            let x = (128 - width) / 2;
            api::canvas_draw_str(x, 41, text);
        }
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
