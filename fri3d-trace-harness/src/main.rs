use fri3d_runtime::canvas::Canvas;
use fri3d_runtime::random::Random;
use fri3d_runtime::trace;
use fri3d_runtime::types::{InputKey, InputType};
use fri3d_runtime::wasm_runner::{ExitToLauncherCallback, StartAppCallback, WasmRunner};
use fri3d_runtime::input::{INPUT_LONG_PRESS_MS, INPUT_REPEAT_INTERVAL_MS, INPUT_REPEAT_START_MS};
use serde::Deserialize;
use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const DEFAULT_FRAME_MS: u32 = 16;
const DEFAULT_SHORT_PRESS_MS: u32 = 10;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mode {
    Fixed,
    Event,
}

#[derive(Debug)]
struct Options {
    wasm_path: PathBuf,
    output_path: PathBuf,
    input_path: Option<PathBuf>,
    app_id: Option<String>,
    frames: u32,
    seed: u32,
    frame_ms: u32,
    scene: Option<u32>,
    mode: Mode,
    duration_ms: Option<u32>,
}

#[derive(Clone, Debug)]
struct RawEvent {
    time_ms: u32,
    key: InputKey,
    kind: InputType,
}

#[derive(Clone, Debug)]
struct TimedEvent {
    time_ms: u32,
    key: InputKey,
    kind: InputType,
}

#[derive(Deserialize)]
struct ScriptEvent {
    time_ms: u32,
    key: String,
    #[serde(rename = "type")]
    kind: String,
    duration_ms: Option<u32>,
}

fn print_usage(program: &str) {
    eprintln!(
        "Usage: {program} --app <path> --out <trace.json> [options]\n\n"
    );
    eprintln!("Options:");
    eprintln!("  --frames <n>       Number of render frames (default: 1)");
    eprintln!("  --seed <n>         RNG seed (default: 42)");
    eprintln!("  --frame-ms <n>     Frame duration in ms (default: 16)");
    eprintln!("  --scene <n>        Set scene (if supported by app)");
    eprintln!("  --input <path>     Input script JSON");
    eprintln!("  --app-id <id>      App id for trace metadata");
    eprintln!("  --mode <fixed|event>   Render schedule (default: fixed)");
    eprintln!("  --duration-ms <n>  Simulation duration for event mode");
    eprintln!("  --help             Show this help");
}

fn parse_args() -> Result<Options, String> {
    let mut args = std::env::args().skip(1);
    let mut wasm_path = None;
    let mut output_path = None;
    let mut input_path = None;
    let mut app_id = None;
    let mut frames = 1u32;
    let mut seed = 42u32;
    let mut frame_ms = DEFAULT_FRAME_MS;
    let mut scene = None;
    let mut mode = Mode::Fixed;
    let mut duration_ms = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--app" => {
                let value = args.next().ok_or("Missing value for --app")?;
                wasm_path = Some(PathBuf::from(value));
            }
            "--out" => {
                let value = args.next().ok_or("Missing value for --out")?;
                output_path = Some(PathBuf::from(value));
            }
            "--input" => {
                let value = args.next().ok_or("Missing value for --input")?;
                input_path = Some(PathBuf::from(value));
            }
            "--frames" => {
                let value = args.next().ok_or("Missing value for --frames")?;
                frames = value.parse().map_err(|_| "Invalid --frames value")?;
            }
            "--seed" => {
                let value = args.next().ok_or("Missing value for --seed")?;
                seed = value.parse().map_err(|_| "Invalid --seed value")?;
            }
            "--frame-ms" => {
                let value = args.next().ok_or("Missing value for --frame-ms")?;
                frame_ms = value.parse().map_err(|_| "Invalid --frame-ms value")?;
            }
            "--scene" => {
                let value = args.next().ok_or("Missing value for --scene")?;
                scene = Some(value.parse().map_err(|_| "Invalid --scene value")?);
            }
            "--app-id" => {
                let value = args.next().ok_or("Missing value for --app-id")?;
                app_id = Some(value);
            }
            "--mode" => {
                let value = args.next().ok_or("Missing value for --mode")?;
                mode = match value.as_str() {
                    "fixed" => Mode::Fixed,
                    "event" => Mode::Event,
                    _ => return Err("Invalid --mode value".to_string()),
                };
            }
            "--duration-ms" => {
                let value = args.next().ok_or("Missing value for --duration-ms")?;
                duration_ms = Some(value.parse().map_err(|_| "Invalid --duration-ms value")?);
            }
            "--help" => {
                return Err("help".to_string());
            }
            other => {
                return Err(format!("Unknown option: {other}"));
            }
        }
    }

    let wasm_path = wasm_path.ok_or("Missing --app")?;
    let output_path = output_path.ok_or("Missing --out")?;

    if frame_ms == 0 {
        frame_ms = DEFAULT_FRAME_MS;
    }

    Ok(Options {
        wasm_path,
        output_path,
        input_path,
        app_id,
        frames,
        seed,
        frame_ms,
        scene,
        mode,
        duration_ms,
    })
}

fn parse_key(value: &str) -> Option<InputKey> {
    match value {
        "up" => Some(InputKey::Up),
        "down" => Some(InputKey::Down),
        "left" => Some(InputKey::Left),
        "right" => Some(InputKey::Right),
        "ok" => Some(InputKey::Ok),
        "back" => Some(InputKey::Back),
        _ => None,
    }
}

fn parse_script_events(path: &Path) -> Result<Vec<ScriptEvent>, String> {
    let text = fs::read_to_string(path).map_err(|err| format!("Failed to open input: {err}"))?;
    let events: Vec<ScriptEvent> = serde_json::from_str(&text)
        .map_err(|err| format!("Invalid input JSON: {err}"))?;
    Ok(events)
}

fn raw_event_weight(kind: InputType) -> i32 {
    match kind {
        InputType::Press => 0,
        InputType::Release => 1,
        _ => 2,
    }
}

fn timed_event_weight(kind: InputType) -> i32 {
    match kind {
        InputType::Press => 0,
        InputType::LongPress | InputType::ShortPress | InputType::Repeat => 1,
        InputType::Release => 2,
    }
}

fn sort_raw_events(events: &mut [RawEvent]) {
    events.sort_by(|a, b| {
        if a.time_ms != b.time_ms {
            return a.time_ms.cmp(&b.time_ms);
        }
        let wa = raw_event_weight(a.kind);
        let wb = raw_event_weight(b.kind);
        if wa != wb {
            return wa.cmp(&wb);
        }
        (a.key as u8).cmp(&(b.key as u8))
    });
}

fn sort_timed_events(events: &mut [TimedEvent]) {
    events.sort_by(|a, b| {
        if a.time_ms != b.time_ms {
            return a.time_ms.cmp(&b.time_ms);
        }
        let wa = timed_event_weight(a.kind);
        let wb = timed_event_weight(b.kind);
        if wa != wb {
            return wa.cmp(&wb);
        }
        (a.key as u8).cmp(&(b.key as u8))
    });
}

fn expand_script_events(script: &[ScriptEvent]) -> Result<Vec<RawEvent>, String> {
    let mut raw_events = Vec::new();

    for event in script {
        let key = parse_key(&event.key)
            .ok_or_else(|| format!("Unknown key: {}", event.key))?;
        let mut duration = event.duration_ms.unwrap_or(0);

        match event.kind.as_str() {
            "press" => raw_events.push(RawEvent {
                time_ms: event.time_ms,
                key,
                kind: InputType::Press,
            }),
            "release" => raw_events.push(RawEvent {
                time_ms: event.time_ms,
                key,
                kind: InputType::Release,
            }),
            "short_press" => {
                if duration == 0 {
                    duration = DEFAULT_SHORT_PRESS_MS;
                }
                raw_events.push(RawEvent {
                    time_ms: event.time_ms,
                    key,
                    kind: InputType::Press,
                });
                raw_events.push(RawEvent {
                    time_ms: event.time_ms + duration,
                    key,
                    kind: InputType::Release,
                });
            }
            "long_press" => {
                if duration < INPUT_LONG_PRESS_MS {
                    duration = INPUT_LONG_PRESS_MS;
                }
                raw_events.push(RawEvent {
                    time_ms: event.time_ms,
                    key,
                    kind: InputType::Press,
                });
                raw_events.push(RawEvent {
                    time_ms: event.time_ms + duration,
                    key,
                    kind: InputType::Release,
                });
            }
            "repeat" => {
                let min_duration = INPUT_REPEAT_START_MS + INPUT_REPEAT_INTERVAL_MS;
                if duration < min_duration {
                    duration = min_duration;
                }
                raw_events.push(RawEvent {
                    time_ms: event.time_ms,
                    key,
                    kind: InputType::Press,
                });
                raw_events.push(RawEvent {
                    time_ms: event.time_ms + duration,
                    key,
                    kind: InputType::Release,
                });
            }
            other => return Err(format!("Unknown input type: {other}")),
        }
    }

    if raw_events.len() > 1 {
        sort_raw_events(&mut raw_events);
    }

    Ok(raw_events)
}

fn build_timed_events(raw_events: &[RawEvent]) -> Vec<TimedEvent> {
    let mut events = Vec::new();
    let mut press_times = [0u32; InputKey::COUNT];
    let mut pressed = [false; InputKey::COUNT];

    for raw in raw_events {
        events.push(TimedEvent {
            time_ms: raw.time_ms,
            key: raw.key,
            kind: raw.kind,
        });

        let key_idx = raw.key as usize;
        match raw.kind {
            InputType::Press => {
                pressed[key_idx] = true;
                press_times[key_idx] = raw.time_ms;
            }
            InputType::Release => {
                if !pressed[key_idx] {
                    continue;
                }
                pressed[key_idx] = false;
                let press_time = press_times[key_idx];
                let release_time = raw.time_ms;
                let hold_time = release_time.saturating_sub(press_time);

                if hold_time >= INPUT_LONG_PRESS_MS {
                    let long_press_time = press_time + INPUT_LONG_PRESS_MS;
                    events.push(TimedEvent {
                        time_ms: long_press_time,
                        key: raw.key,
                        kind: InputType::LongPress,
                    });
                    let mut repeat_time = long_press_time + INPUT_REPEAT_INTERVAL_MS;
                    while repeat_time < release_time {
                        events.push(TimedEvent {
                            time_ms: repeat_time,
                            key: raw.key,
                            kind: InputType::Repeat,
                        });
                        repeat_time += INPUT_REPEAT_INTERVAL_MS;
                    }
                } else {
                    events.push(TimedEvent {
                        time_ms: release_time,
                        key: raw.key,
                        kind: InputType::ShortPress,
                    });
                }
            }
            _ => {}
        }
    }

    if events.len() > 1 {
        sort_timed_events(&mut events);
    }

    events
}

fn resolve_app_id(app_id: Option<String>, wasm_path: &Path) -> String {
    if let Some(app_id) = app_id {
        return app_id;
    }
    wasm_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("app")
        .to_string()
}

fn render_with_request(runner: &mut WasmRunner, frame_index: &mut u32, max_frames: u32) {
    if *frame_index >= max_frames {
        return;
    }
    trace::trace_begin(*frame_index);
    runner.call_render();
    *frame_index += 1;

    if *frame_index >= max_frames {
        return;
    }
    if runner.take_render_request() {
        trace::trace_begin(*frame_index);
        runner.call_render();
        *frame_index += 1;
    }
}

fn run_fixed_mode(
    runner: &mut WasmRunner,
    events: &[TimedEvent],
    frames: u32,
    frame_ms: u32,
    time_cell: &Cell<u32>,
) -> u32 {
    let last_render_time = frames.saturating_sub(1).saturating_mul(frame_ms);
    let last_input_time = events.last().map(|event| event.time_ms).unwrap_or(0);
    let end_time = last_render_time.max(last_input_time);

    let mut frame_index = 0u32;
    let mut next_frame_time = if frames > 0 { 0 } else { u32::MAX };
    let mut next_event_index = 0usize;
    let mut next_event_time = events.first().map(|event| event.time_ms).unwrap_or(u32::MAX);

    loop {
        let next_time = next_frame_time.min(next_event_time);
        if next_time == u32::MAX || next_time > end_time {
            break;
        }

        time_cell.set(next_time);

        if next_time == next_frame_time && frame_index < frames {
            trace::trace_begin(frame_index);
        }

        while next_event_index < events.len() && events[next_event_index].time_ms == next_time {
            let event = &events[next_event_index];
            runner.call_on_input(event.key as u32, event.kind as u32);
            next_event_index += 1;
        }

        if next_time == next_frame_time && frame_index < frames {
            runner.call_render();
            frame_index += 1;
            next_frame_time = if frame_index < frames {
                frame_index.saturating_mul(frame_ms)
            } else {
                u32::MAX
            };
        }

        next_event_time = events
            .get(next_event_index)
            .map(|event| event.time_ms)
            .unwrap_or(u32::MAX);
    }

    frame_index
}

fn run_event_mode(
    runner: &mut WasmRunner,
    events: &[TimedEvent],
    max_frames: u32,
    duration_ms: Option<u32>,
    time_cell: &Cell<u32>,
) -> u32 {
    let mut frame_index = 0u32;
    let mut current_time = 0u32;
    let mut next_event_index = 0usize;

    time_cell.set(0);
    if max_frames > 0 {
        render_with_request(runner, &mut frame_index, max_frames);
    }

    let end_time = duration_ms.unwrap_or_else(|| {
        events.last().map(|event| event.time_ms).unwrap_or(0)
    });

    while frame_index < max_frames {
        let next_input_time = events
            .get(next_event_index)
            .map(|event| event.time_ms)
            .unwrap_or(u32::MAX);
        let next_timer_time = runner.timer_next_ms().unwrap_or(u32::MAX);
        let mut next_time = next_input_time.min(next_timer_time);

        if next_time == u32::MAX {
            break;
        }
        if duration_ms.is_some() && next_time > end_time {
            break;
        }
        current_time = next_time;
        time_cell.set(current_time);

        let mut had_input = false;
        while next_event_index < events.len() && events[next_event_index].time_ms == current_time {
            let event = &events[next_event_index];
            runner.call_on_input(event.key as u32, event.kind as u32);
            had_input = true;
            next_event_index += 1;
        }

        let timer_due = runner.timer_due(current_time);
        if had_input || timer_due {
            render_with_request(runner, &mut frame_index, max_frames);
        }

        if duration_ms.is_some() && current_time >= end_time {
            break;
        }
    }

    frame_index
}

fn main() {
    let options = match parse_args() {
        Ok(options) => options,
        Err(reason) => {
            let program = std::env::args().next().unwrap_or_else(|| "trace_harness".to_string());
            if reason == "help" {
                print_usage(&program);
                std::process::exit(0);
            }
            eprintln!("Error: {reason}");
            print_usage(&program);
            std::process::exit(1);
        }
    };

    let script_events = match options.input_path.as_ref() {
        Some(path) => match parse_script_events(path) {
            Ok(events) => events,
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(1);
            }
        },
        None => Vec::new(),
    };

    let raw_events = match expand_script_events(&script_events) {
        Ok(events) => events,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };
    let timed_events = build_timed_events(&raw_events);

    let canvas = Rc::new(std::cell::RefCell::new(Canvas::new()));
    let random = Rc::new(std::cell::RefCell::new(Random::new(options.seed)));
    let mut runner = WasmRunner::new();
    runner.set_canvas(Rc::clone(&canvas));
    runner.set_random(Rc::clone(&random));

    let time_cell = Cell::new(0u32);
    runner.set_time_provider(|| time_cell.get());

    let exit_cb: ExitToLauncherCallback = Rc::new(|| {});
    let start_cb: StartAppCallback = Rc::new(|_| {});
    runner.set_app_callbacks(exit_cb, start_cb);

    if !runner.load_module(options.wasm_path.to_str().unwrap_or("")) {
        eprintln!("Failed to load module: {}", runner.last_error());
        std::process::exit(1);
    }

    if let Some(scene) = options.scene {
        runner.set_scene(scene);
    }

    trace::trace_reset();

    let rendered_frames = match options.mode {
        Mode::Fixed => run_fixed_mode(&mut runner, &timed_events, options.frames, options.frame_ms, &time_cell),
        Mode::Event => run_event_mode(&mut runner, &timed_events, options.frames, options.duration_ms, &time_cell),
    };

    let app_id = resolve_app_id(options.app_id, &options.wasm_path);
    if let Err(err) = trace::trace_write_json(
        options.output_path.to_str().unwrap_or(""),
        &app_id,
        options.seed,
        rendered_frames,
    ) {
        eprintln!("Failed to write trace output: {err}");
        std::process::exit(1);
    }
}
