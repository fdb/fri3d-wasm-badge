//! fri3d-pack: turn `apps/<id>/{manifest.toml, icon.png, Cargo.toml}` into
//! `build/apps/<id>.fab` bundles, and regenerate `fri3d-apps/src/generated.rs`
//! so hosts can embed them.
//!
//! Usage:
//!   fri3d-pack [--apps-dir apps] [--out build/apps] [--no-build] [--debug]
//!
//! `--no-build` skips `cargo build` and packs whatever wasm is already in
//! `target/`. `--debug` packs the debug build.

use fri3d_kernel::bundle::{self, HeaderBuilder, FLAG_SYSTEM, ICON_H, ICON_LEN, ICON_W};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Deserialize)]
struct Manifest {
    id: String,
    name: String,
    version: String,
    author: String,
    description: String,
    #[serde(default)]
    category: String,
    #[serde(default = "default_icon")]
    icon: String,
    #[serde(default)]
    system: bool,
}

fn default_icon() -> String {
    "icon.png".into()
}

#[derive(Deserialize)]
struct CargoToml {
    package: CargoPackage,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
}

struct Packed {
    id: String,
    system: bool,
    category: String,
    name: String,
    out: PathBuf,
    wasm_len: usize,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("fri3d-pack: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut apps_dir = PathBuf::from("apps");
    let mut out_dir = PathBuf::from("build/apps");
    let mut build = true;
    let mut profile = "release";
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--apps-dir" => apps_dir = args.next().ok_or("--apps-dir needs a value")?.into(),
            "--out" => out_dir = args.next().ok_or("--out needs a value")?.into(),
            "--no-build" => build = false,
            "--debug" => profile = "debug",
            other => return Err(format!("unknown argument {other}")),
        }
    }
    let root = workspace_root()?;
    let apps_dir = root.join(apps_dir);
    let out_dir = root.join(out_dir);
    fs::create_dir_all(&out_dir).map_err(|e| format!("mkdir {}: {e}", out_dir.display()))?;

    let mut dirs: Vec<PathBuf> = fs::read_dir(&apps_dir)
        .map_err(|e| format!("read {}: {e}", apps_dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.join("manifest.toml").is_file())
        .collect();
    dirs.sort();

    if build {
        let crates: Vec<String> = dirs
            .iter()
            .map(|d| crate_name(d))
            .collect::<Result<_, _>>()?;
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&root)
            .arg("build")
            .arg("--target")
            .arg("wasm32-unknown-unknown");
        if profile == "release" {
            cmd.arg("--release");
        }
        for c in &crates {
            cmd.arg("-p").arg(c);
        }
        let status = cmd.status().map_err(|e| format!("cargo: {e}"))?;
        if !status.success() {
            return Err("cargo build failed".into());
        }
    }

    let wasm_opt = which("wasm-opt");
    let mut packed = Vec::new();
    for dir in &dirs {
        packed.push(pack_one(&root, dir, &out_dir, profile, wasm_opt.as_deref())?);
    }

    // Launcher first, then user apps alphabetically, system apps last.
    let launcher = packed
        .iter()
        .position(|p| p.id == "launcher")
        .ok_or("no app with id = \"launcher\"")?;
    let launcher = packed.remove(launcher);
    packed.sort_by(|a, b| (a.system, &a.name).cmp(&(b.system, &b.name)));

    write_generated(&root, &launcher, &packed)?;

    println!("\n{:<14} {:<8} {:>8}  bundle", "id", "category", "wasm");
    for p in std::iter::once(&launcher).chain(packed.iter()) {
        println!(
            "{:<14} {:<8} {:>8}  {}",
            p.id,
            p.category,
            p.wasm_len,
            p.out.strip_prefix(&root).unwrap_or(&p.out).display()
        );
    }
    Ok(())
}

fn pack_one(
    root: &Path,
    dir: &Path,
    out_dir: &Path,
    profile: &str,
    wasm_opt: Option<&Path>,
) -> Result<Packed, String> {
    let manifest_path = dir.join("manifest.toml");
    let manifest: Manifest = toml::from_str(
        &fs::read_to_string(&manifest_path).map_err(|e| format!("{}: {e}", manifest_path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", manifest_path.display()))?;
    validate_id(&manifest.id).map_err(|e| format!("{}: {e}", manifest_path.display()))?;

    let crate_name = crate_name(dir)?;
    let wasm_path = root
        .join("target/wasm32-unknown-unknown")
        .join(profile)
        .join(format!("{}.wasm", crate_name.replace('-', "_")));
    let mut wasm = fs::read(&wasm_path).map_err(|e| format!("{}: {e}", wasm_path.display()))?;

    if let Some(opt) = wasm_opt {
        let tmp = out_dir.join(format!("{}.opt.wasm", manifest.id));
        let status = Command::new(opt)
            .arg("-Oz")
            .arg("--strip-debug")
            .arg("--strip-producers")
            .arg(&wasm_path)
            .arg("-o")
            .arg(&tmp)
            .status()
            .map_err(|e| format!("wasm-opt: {e}"))?;
        if status.success() {
            wasm = fs::read(&tmp).map_err(|e| format!("{}: {e}", tmp.display()))?;
        }
        let _ = fs::remove_file(&tmp);
    }

    let icon_path = dir.join(&manifest.icon);
    let icon = load_icon(&icon_path)?;

    let header = HeaderBuilder::new()
        .id(&manifest.id)
        .name(&manifest.name)
        .version(&manifest.version)
        .author(&manifest.author)
        .description(&manifest.description)
        .flags(if manifest.system { FLAG_SYSTEM } else { 0 })
        .icon(&icon)
        .payload_len(wasm.len() as u32)
        .finish();

    let mut bytes = header.to_vec();
    bytes.extend_from_slice(&wasm);
    bundle::Bundle::parse(&bytes).map_err(|e| format!("{}: produced invalid bundle {e:?}", manifest.id))?;

    let out = out_dir.join(format!("{}.fab", manifest.id));
    fs::write(&out, &bytes).map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(Packed {
        id: manifest.id,
        system: manifest.system,
        category: manifest.category,
        name: manifest.name,
        out,
        wasm_len: wasm.len(),
    })
}

fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() >= bundle::len::ID {
        return Err(format!("id must be 1..{} chars", bundle::len::ID - 1));
    }
    if !id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_') {
        return Err("id must match [a-z0-9_]+".into());
    }
    Ok(())
}

fn crate_name(dir: &Path) -> Result<String, String> {
    let p = dir.join("Cargo.toml");
    let c: CargoToml = toml::from_str(&fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?)
        .map_err(|e| format!("{}: {e}", p.display()))?;
    Ok(c.package.name)
}

/// Decode any PNG into the 14x14 1-bit icon: dark pixels set, light or
/// transparent pixels clear.
fn load_icon(path: &Path) -> Result<[u8; ICON_LEN], String> {
    let file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut decoder = png::Decoder::new(file);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| format!("{}: {e}", path.display()))?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|e| format!("{}: {e}", path.display()))?;
    if info.width as usize != ICON_W || info.height as usize != ICON_H {
        return Err(format!(
            "{}: icon must be {ICON_W}x{ICON_H}, got {}x{}",
            path.display(),
            info.width,
            info.height
        ));
    }
    let channels = match info.color_type {
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        other => return Err(format!("{}: unsupported color type {other:?}", path.display())),
    };
    let mut icon = [0u8; ICON_LEN];
    for y in 0..ICON_H {
        for x in 0..ICON_W {
            let px = &buf[(y * ICON_W + x) * channels..][..channels];
            let (lum, alpha) = match channels {
                1 => (px[0] as u32, 255),
                2 => (px[0] as u32, px[1] as u32),
                3 => ((px[0] as u32 + px[1] as u32 + px[2] as u32) / 3, 255),
                _ => ((px[0] as u32 + px[1] as u32 + px[2] as u32) / 3, px[3] as u32),
            };
            if alpha >= 128 && lum < 128 {
                icon[y * bundle::ICON_ROW_BYTES + x / 8] |= 0x80 >> (x % 8);
            }
        }
    }
    Ok(icon)
}

fn write_generated(root: &Path, launcher: &Packed, apps: &[Packed]) -> Result<(), String> {
    let gen_path = root.join("fri3d-apps/src/generated.rs");
    let rel = |p: &Path| -> String {
        let rel = pathdiff(p, &root.join("fri3d-apps/src"));
        rel.to_string_lossy().replace('\\', "/")
    };
    let mut s = String::new();
    s.push_str("// Generated by tools/fri3d-pack. Do not edit.\n\n");
    s.push_str(&format!(
        "pub static LAUNCHER: &[u8] = include_bytes!(\"{}\");\n\n",
        rel(&launcher.out)
    ));
    s.push_str("pub static APPS: &[&[u8]] = &[\n");
    for p in apps {
        s.push_str(&format!("    include_bytes!(\"{}\"), // {}\n", rel(&p.out), p.id));
    }
    s.push_str("];\n");
    fs::write(&gen_path, s).map_err(|e| format!("{}: {e}", gen_path.display()))?;
    Ok(())
}

fn pathdiff(target: &Path, base: &Path) -> PathBuf {
    let t: Vec<_> = target.components().collect();
    let b: Vec<_> = base.components().collect();
    let common = t.iter().zip(&b).take_while(|(a, b)| a == b).count();
    let mut out = PathBuf::new();
    for _ in common..b.len() {
        out.push("..");
    }
    for c in &t[common..] {
        out.push(c);
    }
    out
}

fn workspace_root() -> Result<PathBuf, String> {
    let out = Command::new("cargo")
        .args(["locate-project", "--workspace", "--message-format=plain"])
        .output()
        .map_err(|e| format!("cargo locate-project: {e}"))?;
    let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Path::new(&p)
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "cannot find workspace root".into())
}

fn which(bin: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|d| d.join(bin))
            .find(|p| p.is_file())
    })
}
