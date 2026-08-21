//! fri3d-pack: turn `apps/<id>/{manifest.toml, icon.png, Cargo.toml}` into
//! `build/apps/<id>.fab` bundles, regenerate `fri3d-apps/src/generated.rs`
//! so hosts can embed them, and regenerate `fri3d-artwork/src/generated.rs`
//! from the system icons in `artwork/icons/`.
//!
//! Icons are PNGs in the DB32 palette: every opaque pixel is snapped to
//! the nearest palette entry, transparent pixels stay holes.
//!
//! Usage:
//!   fri3d-pack [--apps-dir apps] [--out build/apps] [--no-build] [--debug]
//!
//! `--no-build` skips `cargo build` and packs whatever wasm is already in
//! `target/`. `--debug` packs the debug build.

use fri3d_kernel::bundle::{self, HeaderBuilder, FLAG_SYSTEM, ICON_H, ICON_LEN, ICON_W};
use fri3d_kernel::palette;
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
    write_artwork(&root)?;

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

/// Decode the app icon: exactly 16x16, snapped to the DB32 palette.
fn load_icon(path: &Path) -> Result<[u8; ICON_LEN], String> {
    let (w, h, pixels) = load_indexed_png(path)?;
    if w != ICON_W || h != ICON_H {
        return Err(format!("{}: icon must be {ICON_W}x{ICON_H}, got {w}x{h}", path.display()));
    }
    let mut icon = [0u8; ICON_LEN];
    icon.copy_from_slice(&pixels);
    Ok(icon)
}

/// Decode any PNG into DB32 indices (`palette::TRANSPARENT` where alpha < 128).
fn load_indexed_png(path: &Path) -> Result<(usize, usize, Vec<u8>), String> {
    let file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut decoder = png::Decoder::new(file);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| format!("{}: {e}", path.display()))?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|e| format!("{}: {e}", path.display()))?;
    let channels = match info.color_type {
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        other => return Err(format!("{}: unsupported color type {other:?}", path.display())),
    };
    let (w, h) = (info.width as usize, info.height as usize);
    let mut out = Vec::with_capacity(w * h);
    for px in buf[..w * h * channels].chunks(channels) {
        let (r, g, b, a) = match channels {
            1 => (px[0], px[0], px[0], 255),
            2 => (px[0], px[0], px[0], px[1]),
            3 => (px[0], px[1], px[2], 255),
            _ => (px[0], px[1], px[2], px[3]),
        };
        out.push(if a < 128 { palette::TRANSPARENT } else { nearest_db32(r, g, b) });
    }
    Ok((w, h, out))
}

fn nearest_db32(r: u8, g: u8, b: u8) -> u8 {
    let dist = |c: u32| {
        let dr = ((c >> 16) & 0xff) as i32 - r as i32;
        let dg = ((c >> 8) & 0xff) as i32 - g as i32;
        let db = (c & 0xff) as i32 - b as i32;
        dr * dr + dg * dg + db * db
    };
    palette::RGB
        .iter()
        .enumerate()
        .min_by_key(|(_, &c)| dist(c))
        .map(|(i, _)| i as u8)
        .unwrap_or(0)
}

/// `artwork/icons/<name>.png` → `pub static NAME: Image` in fri3d-artwork.
fn write_artwork(root: &Path) -> Result<(), String> {
    let dir = root.join("artwork/icons");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| format!("read {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "png"))
        .collect();
    files.sort();
    let mut s = String::new();
    s.push_str("// Generated by tools/fri3d-pack from artwork/icons/*.png. Do not edit.\n\n");
    s.push_str("use fri3d_wasm_api::Image;\n\n");
    for path in &files {
        let stem = path.file_stem().unwrap().to_string_lossy();
        if !stem.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_') {
            return Err(format!("{}: icon names must match [a-z0-9_]+", path.display()));
        }
        let (w, h, pixels) = load_indexed_png(path)?;
        if w > 64 || h > 64 {
            return Err(format!("{}: system icons are at most 64x64", path.display()));
        }
        s.push_str(&format!("/// {}: {w}x{h}.\n", path.file_name().unwrap().to_string_lossy()));
        s.push_str(&format!(
            "pub static {}: Image = Image {{ w: {w}, h: {h}, pixels: &[\n",
            stem.to_ascii_uppercase()
        ));
        for row in pixels.chunks(w) {
            s.push_str("    ");
            for p in row {
                s.push_str(&format!("{p:>3},"));
            }
            s.push('\n');
        }
        s.push_str("] };\n\n");
    }
    let gen_path = root.join("fri3d-artwork/src/generated.rs");
    fs::write(&gen_path, s).map_err(|e| format!("{}: {e}", gen_path.display()))?;
    Ok(())
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
