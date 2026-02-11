#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;

const HEADER_HEIGHT: i16 = 12;
const PET_RADIUS: i16 = 2;
const MAX_APS: usize = 6;
const ENERGY_MAX: u8 = 100;
const ENERGY_GAIN: u8 = 10;
const TIMER_MS: u32 = 200;

#[derive(Copy, Clone)]
struct AccessPoint {
    x: i16,
    y: i16,
    active: bool,
}

#[derive(Copy, Clone)]
struct State {
    pet_x: i16,
    pet_y: i16,
    energy: u8,
    eaten: u16,
    seed: u32,
    tick: u32,
    initialized: bool,
    wander_dx: i16,
    wander_dy: i16,
    aps: [AccessPoint; MAX_APS],
}

const EMPTY_AP: AccessPoint = AccessPoint {
    x: 0,
    y: 0,
    active: false,
};

const INIT_STATE: State = State {
    pet_x: 0,
    pet_y: 0,
    energy: 60,
    eaten: 0,
    seed: 1,
    tick: 0,
    initialized: false,
    wander_dx: 1,
    wander_dy: 0,
    aps: [EMPTY_AP; MAX_APS],
};

static STATE: api::AppCell<State> = api::AppCell::new(INIT_STATE);

fn render_impl() {
    let width = api::canvas_width() as i16;
    let height = api::canvas_height() as i16;
    if width <= 0 || height <= 0 {
        return;
    }

    let mut state = STATE.get();

    if !state.initialized {
        state.pet_x = width / 2;
        state.pet_y = (height + HEADER_HEIGHT) / 2;
        state.energy = 60;
        state.eaten = 0;
        state.seed = 1;
        state.tick = 0;
        state.wander_dx = 1;
        state.wander_dy = 0;
        state.aps = [EMPTY_AP; MAX_APS];
        state.initialized = true;
        spawn_ap(&mut state, width, height);
        spawn_ap(&mut state, width, height);
        spawn_ap(&mut state, width, height);
        api::start_timer_ms(TIMER_MS);
    }

    state.tick = state.tick.wrapping_add(1);
    update_state(&mut state, width, height);
    draw_scene(&state);

    STATE.set(state);
}

fn on_input_impl(key: u32, kind: u32) {
    let width = api::canvas_width() as i16;
    let height = api::canvas_height() as i16;

    if key == api::input::KEY_BACK
        && (kind == api::input::TYPE_SHORT_PRESS || kind == api::input::TYPE_LONG_PRESS)
    {
        api::stop_timer();
        api::exit_to_launcher();
        return;
    }

    if kind != api::input::TYPE_PRESS && kind != api::input::TYPE_REPEAT {
        return;
    }

    let mut state = STATE.get();
    match key {
        api::input::KEY_OK => {
            spawn_ap(&mut state, width, height);
            state.energy = state.energy.saturating_add(ENERGY_GAIN).min(ENERGY_MAX);
        }
        api::input::KEY_LEFT => state.pet_x -= 2,
        api::input::KEY_RIGHT => state.pet_x += 2,
        api::input::KEY_UP => state.pet_y -= 2,
        api::input::KEY_DOWN => state.pet_y += 2,
        _ => {}
    }
    clamp_pet(&mut state, width, height);
    STATE.set(state);
}

api::export_render!(render_impl);
api::export_on_input!(on_input_impl);
api::wasm_panic_handler!();

fn update_state(state: &mut State, width: i16, height: i16) {
    if state.tick % 5 == 0 {
        state.energy = state.energy.saturating_sub(1);
    }

    let mut target: Option<(i16, i16)> = None;
    let mut best_dist = i32::MAX;
    for ap in state.aps.iter() {
        if !ap.active {
            continue;
        }
        let dx = ap.x - state.pet_x;
        let dy = ap.y - state.pet_y;
        let dist = (dx as i32) * (dx as i32) + (dy as i32) * (dy as i32);
        if dist < best_dist {
            best_dist = dist;
            target = Some((ap.x, ap.y));
        }
    }

    if state.energy > 0 {
        if let Some((tx, ty)) = target {
            state.pet_x += step_towards(state.pet_x, tx);
            state.pet_y += step_towards(state.pet_y, ty);
        } else if state.tick % 8 == 0 {
            pick_wander_direction(state);
            state.pet_x += state.wander_dx;
            state.pet_y += state.wander_dy;
        }
    }

    clamp_pet(state, width, height);

    for ap in state.aps.iter_mut() {
        if !ap.active {
            continue;
        }
        let dx = (ap.x - state.pet_x).abs();
        let dy = (ap.y - state.pet_y).abs();
        if dx <= 2 && dy <= 2 {
            ap.active = false;
            state.eaten = state.eaten.saturating_add(1);
            state.energy = state.energy.saturating_add(ENERGY_GAIN).min(ENERGY_MAX);
        }
    }

    if active_ap_count(state) < 3 && state.tick % 10 == 0 {
        spawn_ap(state, width, height);
    }
}

fn clamp_pet(state: &mut State, width: i16, height: i16) {
    let min_x = PET_RADIUS;
    let max_x = (width - PET_RADIUS - 1).max(min_x);
    let min_y = HEADER_HEIGHT + PET_RADIUS;
    let max_y = (height - PET_RADIUS - 1).max(min_y);
    state.pet_x = clamp_i16(state.pet_x, min_x, max_x);
    state.pet_y = clamp_i16(state.pet_y, min_y, max_y);
}

fn step_towards(current: i16, target: i16) -> i16 {
    if target > current {
        1
    } else if target < current {
        -1
    } else {
        0
    }
}

fn pick_wander_direction(state: &mut State) {
    let dx = rand_range(state, 3) as i16 - 1;
    let dy = rand_range(state, 3) as i16 - 1;
    if dx != 0 || dy != 0 {
        state.wander_dx = dx;
        state.wander_dy = dy;
    }
}

fn spawn_ap(state: &mut State, width: i16, height: i16) {
    let min_x = 2;
    let max_x = (width - 3).max(min_x);
    let min_y = HEADER_HEIGHT + 2;
    let max_y = (height - 3).max(min_y);

    let range_x = (max_x - min_x + 1) as u32;
    let range_y = (max_y - min_y + 1) as u32;

    let x = min_x + rand_range(state, range_x) as i16;
    let y = min_y + rand_range(state, range_y) as i16;

    let slot = match state.aps.iter_mut().find(|ap| !ap.active) {
        Some(slot) => slot,
        None => return,
    };

    slot.x = x;
    slot.y = y;
    slot.active = true;
}

fn active_ap_count(state: &State) -> usize {
    state.aps.iter().filter(|ap| ap.active).count()
}

fn draw_scene(state: &State) {
    api::canvas_set_color(api::color::BLACK);
    api::canvas_set_font(api::font::PRIMARY);

    api::canvas_draw_str(0, 8, "WiFi Pet");
    draw_label_value(0, 16, "Eaten", state.eaten as u32);
    draw_energy_bar(70, 0, 56, 8, state.energy);

    for ap in state.aps.iter() {
        if !ap.active {
            continue;
        }
        api::canvas_draw_frame((ap.x - 1) as i32, (ap.y - 1) as i32, 3, 3);
    }

    let pet_x = state.pet_x as i32;
    let pet_y = state.pet_y as i32;
    api::canvas_draw_disc(pet_x, pet_y, PET_RADIUS as u32);
    api::canvas_draw_dot(pet_x - 1, pet_y - 1);
    api::canvas_draw_dot(pet_x + 1, pet_y - 1);

    if state.energy == 0 {
        api::canvas_draw_line(pet_x - 2, pet_y + 2, pet_x + 2, pet_y + 2);
    }
}

fn draw_energy_bar(x: i32, y: i32, w: u32, h: u32, energy: u8) {
    if w < 2 || h < 2 {
        return;
    }
    api::canvas_draw_frame(x, y, w, h);
    let fill_width = (w - 2) * energy as u32 / ENERGY_MAX as u32;
    if fill_width > 0 {
        api::canvas_draw_box(x + 1, y + 1, fill_width, h - 2);
    }
}

fn draw_label_value(x: i32, y: i32, label: &str, value: u32) {
    api::canvas_draw_str(x, y, label);
    let label_width = api::canvas_string_width(label) as i32;
    draw_number(x + label_width + 2, y, value);
}

fn draw_number(x: i32, y: i32, value: u32) {
    let mut buf = [0u8; 12];
    let len = write_u32(&mut buf, value);
    if let Ok(text) = core::str::from_utf8(&buf[..len]) {
        api::canvas_draw_str(x, y, text);
    }
}

fn write_u32(buf: &mut [u8; 12], mut value: u32) -> usize {
    if value == 0 {
        buf[0] = b'0';
        return 1;
    }

    let mut idx = buf.len();
    while value > 0 {
        let digit = (value % 10) as u8;
        idx -= 1;
        buf[idx] = b'0' + digit;
        value /= 10;
    }

    let len = buf.len() - idx;
    buf.copy_within(idx.., 0);
    len
}

fn clamp_i16(value: i16, min_value: i16, max_value: i16) -> i16 {
    if value < min_value {
        min_value
    } else if value > max_value {
        max_value
    } else {
        value
    }
}

fn rand_range(state: &mut State, max: u32) -> u32 {
    if max == 0 {
        return 0;
    }
    next_rand(state) % max
}

fn next_rand(state: &mut State) -> u32 {
    state.seed = state
        .seed
        .wrapping_mul(1664525)
        .wrapping_add(1013904223);
    state.seed
}
