# Visual Regression Testing

Visual regression tests for FRI3D WASM Badge apps.

## Quick Start

```bash
# Run all tests
uv run tests/visual/run_visual_tests.py

# Test a specific app
uv run tests/visual/run_visual_tests.py --app test_drawing

# Update golden images after intentional changes
uv run tests/visual/run_visual_tests.py --update-golden
```

## Directory Structure

```
tests/visual/
├── apps/                    # App test configurations
│   ├── test_drawing/
│   │   ├── config.yaml      # App metadata and scene list
│   │   └── golden/          # Reference images (0.png, 1.png, ...)
│   ├── circles/
│   │   ├── config.yaml
│   │   └── golden/
│   └── mandelbrot/
│       ├── config.yaml
│       └── golden/
├── output/                  # Test output (gitignored)
├── reports/                 # HTML reports (gitignored)
└── run_visual_tests.py      # Test runner (uses uv inline dependencies)
```

## Adding a New App

1. Create `tests/visual/apps/{app_name}/config.yaml`:

```yaml
name: My App
description: What this app does

scenes:
  - name: Default View
  - name: Another Scene
```

2. Add scene control exports to your app's C code:

```c
uint32_t get_scene(void) { return current_scene; }
void set_scene(uint32_t scene) { current_scene = scene; }
uint32_t get_scene_count(void) { return SCENE_COUNT; }
```

3. Generate golden images:

```bash
uv run tests/visual/run_visual_tests.py --app my_app --update-golden
```

## Config Format

Each `config.yaml` defines the app name and its scenes:

```yaml
name: Human Readable Name
description: Brief description of what this app tests

scenes:
  - name: Scene Zero    # Index 0 → golden/0.png
  - name: Scene One     # Index 1 → golden/1.png
  - name: Scene Two     # Index 2 → golden/2.png
```

Scene indices are implicit based on position in the list.

## Prerequisites

Build the host emulator and apps before running tests:

```bash
# Host emulator
cmake -B build && cmake --build build

# All apps
for app in test_drawing circles mandelbrot; do
  cmake -B build-apps/$app \
    -DCMAKE_TOOLCHAIN_FILE=cmake/toolchain-wasm.cmake \
    -S src/apps/$app
  cmake --build build-apps/$app
done
```

## Diff Legend

The HTML report shows three images per test:

| Image | Description |
|-------|-------------|
| Expected | Golden reference image |
| Actual | Current output |
| Diff | Visual comparison |

Diff colors:
- **Green**: Pixels added (in actual but not golden)
- **Red**: Pixels removed (in golden but not actual)
- **Dark gray**: Unchanged black pixels
- **Light gray**: Unchanged white pixels

## CI Integration

The script exits with code 1 on failures, making it CI-ready.
Dependencies are embedded via PEP 723 inline script metadata—just run with `uv`.
