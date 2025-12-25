# Visual Regression Testing

This directory contains the visual regression testing infrastructure for the FRI3D WASM Badge project.

## Overview

The visual regression testing system allows you to:
- Capture screenshots of drawing primitives rendered by the test_drawing app
- Compare screenshots against "golden" reference images
- Generate detailed HTML reports showing expected, actual, and diff images
- Detect pixel-level differences (green = added pixels, red = removed pixels)

## Directory Structure

```
tests/visual/
├── golden/           # Reference screenshots (committed to git)
├── actual/           # Generated during test runs (gitignored)
├── diff/             # Diff images (gitignored)
├── reports/          # HTML test reports (gitignored)
├── run_visual_tests.py   # Main test runner script
├── requirements.txt      # Python dependencies
└── README.md            # This file
```

## Prerequisites

1. **Host emulator built:**
   ```bash
   cmake -B build
   cmake --build build
   ```

2. **test_drawing WASM app built:**
   ```bash
   cmake -B build-apps/test_drawing \
     -DCMAKE_TOOLCHAIN_FILE=cmake/toolchain-wasm.cmake \
     -S src/apps/test_drawing
   cmake --build build-apps/test_drawing
   ```

3. **Python 3 with Pillow:**
   ```bash
   pip install -r tests/visual/requirements.txt
   ```

## Usage

### Run All Tests
```bash
python tests/visual/run_visual_tests.py
```

### Update Golden Screenshots
When you intentionally change the rendering output, update the golden screenshots:
```bash
python tests/visual/run_visual_tests.py --update-golden
```

### Test a Single Scene
```bash
python tests/visual/run_visual_tests.py --scene 0
```

### Skip Report Generation
```bash
python tests/visual/run_visual_tests.py --no-report
```

## Test Scenes

The test_drawing app includes the following scenes:

| ID | Name | Description |
|----|------|-------------|
| 0 | horizontal_lines | Tests horizontal line rendering |
| 1 | vertical_lines | Tests vertical line rendering |
| 2 | diagonal_lines | Tests diagonal line rendering |
| 3 | random_pixels | Tests pixel plotting with seeded random |
| 4 | circles | Tests circle outlines |
| 5 | filled_circles | Tests filled circles (discs) |
| 6 | rectangles | Tests rectangle outlines |
| 7 | filled_rectangles | Tests filled rectangles |
| 8 | rounded_rectangles | Tests rounded rectangles |
| 9 | text_rendering | Tests text rendering with different fonts |
| 10 | mixed_primitives | Tests combination of primitives |
| 11 | checkerboard | Tests checkerboard pattern |

## Interpreting Results

The HTML report shows three images for each test:

- **Expected (Golden):** The reference image that represents correct output
- **Actual:** The screenshot captured during the test run
- **Diff:** Highlights differences between expected and actual:
  - **Green pixels:** Present in actual but not in golden (added)
  - **Red pixels:** Present in golden but not in actual (removed)
  - **Dark gray:** Unchanged black pixels
  - **Light gray:** Unchanged white pixels

## Deterministic Output

The test_drawing app uses a fixed random seed (12345) to ensure deterministic output. This means:
- Random pixel positions are always the same
- Tests can reliably compare pixel-by-pixel
- No anti-aliasing is used, ensuring pixel-perfect matching

## CI Integration

The test runner returns exit code 0 if all tests pass, and exit code 1 if any test fails. This makes it suitable for CI integration:

```yaml
# Example GitHub Actions step
- name: Run Visual Tests
  run: python tests/visual/run_visual_tests.py
```

## Host Emulator Options

The host emulator supports several options for testing:

```
host_emulator [options] <wasm_file>

Options:
  --test              Run in test mode (render and exit)
  --scene <n>         Set scene number (for test_drawing app)
  --screenshot <path> Save screenshot to path (PPM format)
  --headless          Run without display (requires --screenshot)
  --help              Show help
```

Example: Capture a screenshot of scene 3:
```bash
./build/src/host/host_emulator \
  --headless \
  --scene 3 \
  --screenshot output.ppm \
  build-apps/test_drawing/test_drawing.wasm
```

## Interactive Mode

Press 'S' during interactive mode to save a screenshot to the current directory.

Use arrow keys to cycle through scenes in the test_drawing app.
