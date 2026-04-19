# Visual Regression Testing

Pixel-diffs the C++ firmware's rendering against Rust-generated golden PNGs
under `apps/<name>/golden/<scene>.png`. Any mismatch means the C++ port of
a canvas primitive, the u8g2 font decoder, or a host function has drifted
from the Rust reference in `fri3d-runtime`.

## Quick start

```bash
# From the repo root:
firmware/tools/app-runner/visual_parity.sh
```

Outputs `[pass]` / `[FAIL]` per app, plus a summary. On failure, the C++
render is saved to `/tmp/<app>_cpp.png` for inspection alongside the
`apps/<app>/golden/0.png` reference.

## Directory structure

```
tests/visual/
├── apps/
│   ├── circles/
│   │   ├── config.yaml      # App metadata (historical, not read by
│   │   │                    # visual_parity.sh — kept for reference)
│   │   └── golden/
│   │       └── 0.png        # 128×64 mono, Rust-generated
│   ├── snake/
│   ├── launcher/
│   └── ...
```

## Adding a new golden

`visual_parity.sh` currently always runs `--frames 1 --seed 42`. If you
add a new app or scene, you'll need a Rust-generated golden to diff
against. Since `fri3d-emulator` is retired, the recommended path is:

1. Temporarily re-add a golden-generator that uses `fri3d-runtime`, or
2. Render once via `firmware/tools/app-runner/app_runner` and commit the
   output as the provisional golden — this makes the C++ side self-
   referential for that app; OK as long as canvas parity tests keep the
   primitives locked to Rust.

## CI

`.github/workflows/parity.yml` runs this on every push to `main` plus
every PR. Failures upload the C++ renders as artifacts.
