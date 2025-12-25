#!/usr/bin/env python3
"""
Generate golden screenshots for visual regression testing.

This script captures screenshots from the test_drawing app and saves them
as reference images for visual regression tests.

Usage:
    python scripts/update_golden_screenshots.py

Prerequisites:
    1. Build the host emulator:
       cmake -B build && cmake --build build

    2. Build the test_drawing app:
       cmake -B build-apps/test_drawing \
         -DCMAKE_TOOLCHAIN_FILE=cmake/toolchain-wasm.cmake \
         -S src/apps/test_drawing
       cmake --build build-apps/test_drawing

    3. Install Python dependencies:
       pip install Pillow
"""

import subprocess
import sys
from pathlib import Path

# Paths
SCRIPT_DIR = Path(__file__).parent.absolute()
PROJECT_ROOT = SCRIPT_DIR.parent
HOST_EMULATOR = PROJECT_ROOT / "build" / "src" / "host" / "host_emulator"
GOLDEN_DIR = PROJECT_ROOT / "tests" / "visual" / "golden"

# Scene definitions matching test_drawing/main.c
SCENES = [
    "horizontal_lines",
    "vertical_lines",
    "diagonal_lines",
    "random_pixels",
    "circles",
    "filled_circles",
    "rectangles",
    "filled_rectangles",
    "rounded_rectangles",
    "text_rendering",
    "mixed_primitives",
    "checkerboard",
]


def find_test_app() -> Path | None:
    """Find the test_drawing WASM app."""
    build_dir = PROJECT_ROOT / "build-apps" / "test_drawing"
    candidates = [
        build_dir / "test_drawing.wasm",
        build_dir / "test_drawing",
    ]

    if build_dir.exists():
        for f in build_dir.glob("*.wasm"):
            candidates.append(f)

    for path in candidates:
        if path.exists():
            return path
    return None


def main():
    # Check prerequisites
    if not HOST_EMULATOR.exists():
        print(f"Error: Host emulator not found at {HOST_EMULATOR}")
        print("Build it with: cmake -B build && cmake --build build")
        sys.exit(1)

    test_app = find_test_app()
    if test_app is None:
        print("Error: test_drawing app not found")
        print("Build it with:")
        print("  cmake -B build-apps/test_drawing \\")
        print("    -DCMAKE_TOOLCHAIN_FILE=cmake/toolchain-wasm.cmake \\")
        print("    -S src/apps/test_drawing")
        print("  cmake --build build-apps/test_drawing")
        sys.exit(1)

    print(f"Using test app: {test_app}")
    print(f"Saving golden screenshots to: {GOLDEN_DIR}")
    print()

    # Create golden directory
    GOLDEN_DIR.mkdir(parents=True, exist_ok=True)

    # Generate screenshots for each scene
    for scene_id, scene_name in enumerate(SCENES):
        output_path = GOLDEN_DIR / f"{scene_name}.ppm"
        print(f"  [{scene_id + 1}/{len(SCENES)}] {scene_name}...", end=" ", flush=True)

        cmd = [
            str(HOST_EMULATOR),
            "--headless",
            "--scene", str(scene_id),
            "--screenshot", str(output_path),
            str(test_app)
        ]

        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
            if result.returncode != 0:
                print(f"FAILED")
                print(f"    Error: {result.stderr}")
                continue
            if output_path.exists():
                print("OK")
            else:
                print("FAILED (no output)")
        except subprocess.TimeoutExpired:
            print("FAILED (timeout)")
        except Exception as e:
            print(f"FAILED ({e})")

    print()
    print("Done! Now commit the golden screenshots:")
    print(f"  git add {GOLDEN_DIR.relative_to(PROJECT_ROOT)}/")
    print("  git commit -m 'Add golden screenshots for visual regression tests'")


if __name__ == "__main__":
    main()
