#[derive(Clone, Debug)]
pub enum TraceArg {
    Int(i64),
    Str(String),
}

impl TraceArg {
    pub fn int(value: i64) -> Self {
        TraceArg::Int(value)
    }

    pub fn str(value: &str) -> Self {
        TraceArg::Str(value.to_string())
    }
}

#[cfg(feature = "trace")]
mod enabled {
    use super::{TraceArg};
    use std::fmt::Write as _;
    use std::fs::File;
    use std::io::{self, Write};
    use std::sync::{Mutex, OnceLock};

    #[derive(Clone, Debug)]
    struct TraceEvent {
        frame: u32,
        fn_name: String,
        args: Vec<TraceArg>,
        ret: Option<i64>,
    }

    #[derive(Default)]
    struct TraceState {
        events: Vec<TraceEvent>,
        current_frame: u32,
    }

    static TRACE_STATE: OnceLock<Mutex<TraceState>> = OnceLock::new();

    fn state() -> &'static Mutex<TraceState> {
        TRACE_STATE.get_or_init(|| Mutex::new(TraceState::default()))
    }

    pub fn trace_reset() {
        let mut guard = state().lock().unwrap();
        guard.events.clear();
        guard.current_frame = 0;
    }

    pub fn trace_begin(frame: u32) {
        let mut guard = state().lock().unwrap();
        guard.current_frame = frame;
    }

    pub fn trace_call(fn_name: &str, args: &[TraceArg]) {
        let mut guard = state().lock().unwrap();
        guard.events.push(TraceEvent {
            frame: guard.current_frame,
            fn_name: fn_name.to_string(),
            args: args.to_vec(),
            ret: None,
        });
    }

    pub fn trace_result(value: i64) {
        let mut guard = state().lock().unwrap();
        if let Some(last) = guard.events.last_mut() {
            last.ret = Some(value);
        }
    }

    fn write_json_string(mut out: &mut String, value: &str) {
        out.push('"');
        for c in value.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c.is_control() => {
                    let _ = write!(out, "\\u{:04x}", c as u32);
                }
                c => out.push(c),
            }
        }
        out.push('"');
    }

    pub fn trace_write_json(path: &str, app: &str, seed: u32, frames: u32) -> io::Result<()> {
        let guard = state().lock().unwrap();
        let mut output = String::new();
        output.push_str("{\n");
        output.push_str("  \"app\": ");
        write_json_string(&mut output, app);
        let _ = write!(
            output,
            ",\n  \"seed\": {},\n  \"frames\": {},\n  \"events\": [\n",
            seed, frames
        );

        for (idx, event) in guard.events.iter().enumerate() {
            output.push_str("    {\"frame\": ");
            let _ = write!(output, "{}", event.frame);
            output.push_str(", \"fn\": ");
            write_json_string(&mut output, &event.fn_name);
            output.push_str(", \"args\": [");
            for (arg_idx, arg) in event.args.iter().enumerate() {
                if arg_idx > 0 {
                    output.push_str(", ");
                }
                match arg {
                    TraceArg::Int(value) => {
                        let _ = write!(output, "{}", value);
                    }
                    TraceArg::Str(value) => {
                        write_json_string(&mut output, value);
                    }
                }
            }
            output.push(']');
            if let Some(ret) = event.ret {
                let _ = write!(output, ", \"ret\": {}", ret);
            }
            output.push('}');
            if idx + 1 < guard.events.len() {
                output.push(',');
            }
            output.push('\n');
        }

        output.push_str("  ]\n}\n");
        let mut file = File::create(path)?;
        file.write_all(output.as_bytes())
    }
}

#[cfg(not(feature = "trace"))]
mod disabled {
    use super::TraceArg;

    pub fn trace_reset() {}
    pub fn trace_begin(_frame: u32) {}
    pub fn trace_call(_fn_name: &str, _args: &[TraceArg]) {}
    pub fn trace_result(_value: i64) {}
    pub fn trace_write_json(
        _path: &str,
        _app: &str,
        _seed: u32,
        _frames: u32,
    ) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "trace")]
pub use enabled::*;
#[cfg(not(feature = "trace"))]
pub use disabled::*;
