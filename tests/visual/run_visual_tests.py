#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "pillow >= 9",
#   "pyyaml >= 6",
# ]
# ///
"""
Visual Regression Testing for FRI3D WASM Badge

Tests all apps by capturing screenshots and comparing against golden images.

Usage:
    uv run tests/visual/run_visual_tests.py                 # Test all apps
    uv run tests/visual/run_visual_tests.py --app circles   # Test specific app
    uv run tests/visual/run_visual_tests.py --update-golden # Update golden images
"""

from __future__ import annotations

import argparse
import base64
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime
from io import BytesIO
from pathlib import Path
from typing import Optional

import yaml
from PIL import Image


# =============================================================================
# Configuration
# =============================================================================

SCRIPT_DIR = Path(__file__).parent.absolute()
PROJECT_ROOT = SCRIPT_DIR.parent.parent
DEFAULT_EMULATOR_DEBUG = PROJECT_ROOT / "target" / "debug" / "fri3d-emulator"
DEFAULT_EMULATOR_RELEASE = PROJECT_ROOT / "target" / "release" / "fri3d-emulator"
HOST_EMULATOR = Path(
    os.environ.get("FRI3D_EMULATOR", DEFAULT_EMULATOR_DEBUG)
)
if not HOST_EMULATOR.exists() and DEFAULT_EMULATOR_RELEASE.exists():
    HOST_EMULATOR = DEFAULT_EMULATOR_RELEASE
APPS_DIR = SCRIPT_DIR / "apps"
OUTPUT_DIR = SCRIPT_DIR / "output"
REPORTS_DIR = SCRIPT_DIR / "reports"


# =============================================================================
# Data Structures
# =============================================================================

@dataclass
class Scene:
    """A single test scene within an app."""
    index: int
    name: str


@dataclass
class App:
    """An app configuration loaded from config.yaml."""
    id: str
    name: str
    description: str
    scenes: list[Scene]
    path: Path

    @classmethod
    def load(cls, app_dir: Path) -> App:
        """Load app configuration from a directory."""
        config_path = app_dir / "config.yaml"
        with open(config_path) as f:
            config = yaml.safe_load(f)

        scenes = [
            Scene(index=i, name=scene["name"])
            for i, scene in enumerate(config.get("scenes", []))
        ]

        return cls(
            id=app_dir.name,
            name=config.get("name", app_dir.name),
            description=config.get("description", ""),
            scenes=scenes,
            path=app_dir,
        )


@dataclass
class TestResult:
    """Result of testing a single scene."""
    app: App
    scene: Scene
    passed: bool
    diff_count: int
    golden_path: Optional[Path]
    actual_path: Optional[Path]
    diff_path: Optional[Path]
    error: Optional[str]


# =============================================================================
# App Discovery
# =============================================================================

def discover_apps() -> list[App]:
    """Find all apps with visual tests configured."""
    apps = []
    for app_dir in sorted(APPS_DIR.iterdir()):
        if app_dir.is_dir() and (app_dir / "config.yaml").exists():
            apps.append(App.load(app_dir))
    return apps


def find_wasm_binary(app_id: str) -> Optional[Path]:
    """Find the compiled WASM binary for an app."""
    rust_name = f"fri3d_app_{app_id.replace('-', '_')}"
    for profile in ["debug", "release"]:
        candidate = PROJECT_ROOT / "target" / "wasm32-unknown-unknown" / profile / f"{rust_name}.wasm"
        if candidate.exists():
            return candidate

    build_dir = PROJECT_ROOT / "build" / "apps" / app_id

    if not build_dir.exists():
        return None

    # Check for .wasm file first, then without extension
    for pattern in [f"{app_id}.wasm", app_id]:
        candidate = build_dir / pattern
        if candidate.exists():
            return candidate

    # Search for any .wasm file
    wasm_files = list(build_dir.glob("*.wasm"))
    if wasm_files:
        return wasm_files[0]

    return None


# =============================================================================
# Screenshot Capture
# =============================================================================

def capture_screenshot(app_id: str, scene_index: int, output_path: Path) -> bool:
    """Capture a screenshot from the host emulator."""
    wasm_path = find_wasm_binary(app_id)
    if not wasm_path:
        print(f"    Error: WASM binary not found for {app_id}")
        return False

    cmd = [
        str(HOST_EMULATOR),
        "--headless",
        "--scene", str(scene_index),
        "--screenshot", str(output_path),
        str(wasm_path),
    ]

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        if result.returncode != 0:
            print(f"    Error: {result.stderr.strip()}")
            return False
        return output_path.exists()
    except subprocess.TimeoutExpired:
        print(f"    Error: Screenshot capture timed out")
        return False
    except Exception as e:
        print(f"    Error: {e}")
        return False


# =============================================================================
# Image Comparison
# =============================================================================

def compare_images(golden: Image.Image, actual: Image.Image) -> tuple[int, Image.Image]:
    """
    Compare two images and generate a diff visualization.

    Returns (diff_count, diff_image) where:
    - diff_count: number of pixels that differ
    - diff_image: RGB visualization (green=added, red=removed, gray=unchanged)
    """
    if golden.size != actual.size:
        raise ValueError(f"Size mismatch: {golden.size} vs {actual.size}")

    golden_gray = golden.convert("L")
    actual_gray = actual.convert("L")

    width, height = golden.size
    diff_img = Image.new("RGB", (width, height))
    diff_pixels = diff_img.load()
    golden_pixels = golden_gray.load()
    actual_pixels = actual_gray.load()

    diff_count = 0
    threshold = 128

    for y in range(height):
        for x in range(width):
            g_black = golden_pixels[x, y] < threshold
            a_black = actual_pixels[x, y] < threshold

            if g_black == a_black:
                gray = 50 if g_black else 200
                diff_pixels[x, y] = (gray, gray, gray)
            elif a_black:
                diff_pixels[x, y] = (0, 255, 0)  # Added (green)
                diff_count += 1
            else:
                diff_pixels[x, y] = (255, 0, 0)  # Removed (red)
                diff_count += 1

    return diff_count, diff_img


# =============================================================================
# Test Execution
# =============================================================================

def run_scene_test(app: App, scene: Scene, update_golden: bool = False) -> TestResult:
    """Run visual test for a single scene."""
    golden_dir = app.path / "golden"
    actual_dir = OUTPUT_DIR / app.id / "actual"
    diff_dir = OUTPUT_DIR / app.id / "diff"

    golden_path = golden_dir / f"{scene.index}.png"
    actual_path = actual_dir / f"{scene.index}.png"
    diff_path = diff_dir / f"{scene.index}.png"

    actual_dir.mkdir(parents=True, exist_ok=True)
    diff_dir.mkdir(parents=True, exist_ok=True)

    # Capture screenshot
    if not capture_screenshot(app.id, scene.index, actual_path):
        return TestResult(
            app=app, scene=scene, passed=False, diff_count=-1,
            golden_path=None, actual_path=None, diff_path=None,
            error="Failed to capture screenshot",
        )

    # Update golden if requested
    if update_golden:
        golden_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy(actual_path, golden_path)
        return TestResult(
            app=app, scene=scene, passed=True, diff_count=0,
            golden_path=golden_path, actual_path=actual_path, diff_path=None,
            error=None,
        )

    # Check golden exists
    if not golden_path.exists():
        return TestResult(
            app=app, scene=scene, passed=False, diff_count=-1,
            golden_path=None, actual_path=actual_path, diff_path=None,
            error="Golden image not found (run with --update-golden)",
        )

    # Compare images
    try:
        golden_img = Image.open(golden_path)
        actual_img = Image.open(actual_path)
        diff_count, diff_img = compare_images(golden_img, actual_img)
        diff_img.save(diff_path)

        return TestResult(
            app=app, scene=scene, passed=(diff_count == 0), diff_count=diff_count,
            golden_path=golden_path, actual_path=actual_path, diff_path=diff_path,
            error=None,
        )
    except Exception as e:
        return TestResult(
            app=app, scene=scene, passed=False, diff_count=-1,
            golden_path=golden_path, actual_path=actual_path, diff_path=None,
            error=str(e),
        )


def run_app_tests(app: App, update_golden: bool = False) -> list[TestResult]:
    """Run all visual tests for an app."""
    print(f"\n{app.name}")
    print("=" * len(app.name))

    results = []
    for scene in app.scenes:
        print(f"  [{scene.index}] {scene.name}...", end=" ", flush=True)
        result = run_scene_test(app, scene, update_golden)
        results.append(result)

        if update_golden:
            print("UPDATED")
        elif result.passed:
            print("PASSED")
        elif result.error:
            print(f"FAILED ({result.error})")
        else:
            print(f"FAILED ({result.diff_count} pixels differ)")

    return results


# =============================================================================
# HTML Report Generation
# =============================================================================

def image_to_data_uri(path: Optional[Path], scale: int = 4) -> str:
    """Convert an image to a base64 data URI, scaled up for visibility."""
    if not path or not path.exists():
        return ""

    try:
        img = Image.open(path)
        img = img.resize((img.width * scale, img.height * scale), Image.NEAREST)
        buffer = BytesIO()
        img.save(buffer, format="PNG")
        b64 = base64.b64encode(buffer.getvalue()).decode()
        return f"data:image/png;base64,{b64}"
    except Exception:
        return ""


def generate_html_report(results: list[TestResult], output_path: Path):
    """Generate an HTML test report."""
    passed = sum(1 for r in results if r.passed)
    failed = len(results) - passed

    # Group results by app
    apps: dict[str, list[TestResult]] = {}
    for r in results:
        apps.setdefault(r.app.id, []).append(r)

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Visual Regression Test Report</title>
    <style>
        :root {{ --pass: #22c55e; --fail: #ef4444; --bg: #f5f5f5; --card: white; }}
        * {{ box-sizing: border-box; }}
        body {{ font-family: system-ui, sans-serif; max-width: 1400px; margin: 0 auto; padding: 20px; background: var(--bg); }}
        h1 {{ color: #333; border-bottom: 2px solid #333; padding-bottom: 10px; }}
        h2 {{ color: #555; margin-top: 2em; }}
        .summary {{ background: var(--card); padding: 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .stats {{ display: flex; gap: 15px; margin-top: 10px; }}
        .stat {{ padding: 8px 16px; border-radius: 4px; font-weight: 600; }}
        .stat.pass {{ background: #dcfce7; color: #166534; }}
        .stat.fail {{ background: #fee2e2; color: #991b1b; }}
        .legend {{ background: var(--card); padding: 15px 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); display: flex; gap: 25px; flex-wrap: wrap; }}
        .legend-item {{ display: flex; align-items: center; gap: 8px; }}
        .legend-color {{ width: 18px; height: 18px; border-radius: 3px; border: 1px solid #ccc; }}
        .test {{ background: var(--card); padding: 20px; border-radius: 8px; margin-bottom: 15px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); border-left: 4px solid var(--pass); }}
        .test.fail {{ border-left-color: var(--fail); }}
        .test-header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px; }}
        .test-title {{ font-size: 1.1em; font-weight: 600; }}
        .badge {{ padding: 4px 12px; border-radius: 20px; font-size: 0.85em; font-weight: 600; color: white; }}
        .badge.pass {{ background: var(--pass); }}
        .badge.fail {{ background: var(--fail); }}
        .images {{ display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; }}
        .image-box {{ text-align: center; }}
        .image-box h4 {{ margin: 0 0 8px; color: #666; font-size: 0.9em; }}
        .image-box img {{ border: 1px solid #ddd; image-rendering: pixelated; max-width: 100%; }}
        .error {{ background: #fef3c7; color: #92400e; padding: 10px 15px; border-radius: 6px; margin-top: 10px; }}
        .diff-info {{ color: #666; font-size: 0.9em; margin-left: 10px; }}
    </style>
</head>
<body>
    <h1>Visual Regression Test Report</h1>
    <div class="summary">
        <strong>Generated:</strong> {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
        <div class="stats">
            <span class="stat pass">{passed} Passed</span>
            <span class="stat fail">{failed} Failed</span>
        </div>
    </div>
    <div class="legend">
        <div class="legend-item"><div class="legend-color" style="background:#0f0"></div><span>Added pixels</span></div>
        <div class="legend-item"><div class="legend-color" style="background:#f00"></div><span>Removed pixels</span></div>
        <div class="legend-item"><div class="legend-color" style="background:#323232"></div><span>Unchanged (black)</span></div>
        <div class="legend-item"><div class="legend-color" style="background:#c8c8c8"></div><span>Unchanged (white)</span></div>
    </div>
"""

    for app_id, app_results in apps.items():
        app = app_results[0].app
        html += f'    <h2>{app.name}</h2>\n'

        for r in app_results:
            status = "pass" if r.passed else "fail"
            html += f"""    <div class="test {status}">
        <div class="test-header">
            <span class="test-title">[{r.scene.index}] {r.scene.name}</span>
            <span>
                <span class="badge {status}">{"PASSED" if r.passed else "FAILED"}</span>
                {"<span class='diff-info'>" + str(r.diff_count) + " pixels</span>" if r.diff_count > 0 else ""}
            </span>
        </div>
"""
            if r.error:
                html += f'        <div class="error">{r.error}</div>\n'
            else:
                golden_uri = image_to_data_uri(r.golden_path)
                actual_uri = image_to_data_uri(r.actual_path)
                diff_uri = image_to_data_uri(r.diff_path)
                html += f"""        <div class="images">
            <div class="image-box"><h4>Expected</h4>{"<img src='" + golden_uri + "'>" if golden_uri else "<p>N/A</p>"}</div>
            <div class="image-box"><h4>Actual</h4>{"<img src='" + actual_uri + "'>" if actual_uri else "<p>N/A</p>"}</div>
            <div class="image-box"><h4>Diff</h4>{"<img src='" + diff_uri + "'>" if diff_uri else "<p>N/A</p>"}</div>
        </div>
"""
            html += "    </div>\n"

    html += "</body>\n</html>\n"
    output_path.write_text(html)


# =============================================================================
# Prerequisites Check
# =============================================================================

def check_prerequisites() -> bool:
    """Verify host emulator exists."""
    if not HOST_EMULATOR.exists():
        print(f"Error: Emulator not found at {HOST_EMULATOR}")
        print("Build with: cargo build -p fri3d-emulator")
        print("Or set FRI3D_EMULATOR to the emulator path")
        return False

    return True


def filter_apps_with_wasm(apps: list[App]) -> tuple[list[App], list[str]]:
    """Return apps with WASM binaries and a list of missing app ids."""
    available = []
    missing = []
    for app in apps:
        if find_wasm_binary(app.id):
            available.append(app)
        else:
            missing.append(app.id)
    return available, missing


def has_golden_images(app: App) -> bool:
    """Check if an app has golden images."""
    golden_dir = app.path / "golden"
    return golden_dir.exists() and list(golden_dir.glob("*.png"))


def has_any_golden_images(apps: list[App]) -> bool:
    """Check if any golden images exist."""
    return any(has_golden_images(app) for app in apps)


# =============================================================================
# Main Entry Point
# =============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="Visual regression testing for FRI3D WASM Badge apps"
    )
    parser.add_argument(
        "--app", metavar="NAME",
        help="Test specific app only (e.g., test_drawing, circles, mandelbrot)"
    )
    parser.add_argument(
        "--update-golden", action="store_true",
        help="Update golden images with current output"
    )
    parser.add_argument(
        "--no-report", action="store_true",
        help="Skip HTML report generation"
    )
    args = parser.parse_args()

    # Discover apps
    all_apps = discover_apps()
    if not all_apps:
        print("Error: No apps found in tests/visual/apps/")
        sys.exit(1)

    # Filter to specific app if requested
    if args.app:
        apps = [a for a in all_apps if a.id == args.app]
        if not apps:
            print(f"Error: App '{args.app}' not found")
            print(f"Available: {', '.join(a.id for a in all_apps)}")
            sys.exit(1)
    else:
        apps = all_apps

    # Check prerequisites
    if not check_prerequisites():
        sys.exit(1)

    # Filter apps based on available WASM binaries
    apps, missing_apps = filter_apps_with_wasm(apps)
    if args.app and missing_apps:
        print(f"Error: WASM binary not found for {missing_apps[0]}")
        sys.exit(1)
    if missing_apps and not args.app:
        print(f"Skipping apps without WASM binaries: {', '.join(missing_apps)}")

    # Warn if no golden images (unless updating)
    if not args.update_golden and not has_any_golden_images(apps):
        print("\nNo golden images found. Run with --update-golden first.")
        sys.exit(0)

    # Run tests
    print(f"\nTesting {len(apps)} app(s): {', '.join(a.id for a in apps)}")

    all_results = []
    skipped_apps = []

    for app in apps:
        # Skip apps without golden images (unless updating)
        if not args.update_golden and not has_golden_images(app):
            skipped_apps.append(app.id)
            continue

        results = run_app_tests(app, args.update_golden)
        all_results.extend(results)

    if skipped_apps:
        print(f"\nSkipped (no golden images): {', '.join(skipped_apps)}")
        print("  Run with --update-golden to generate them")

    # Generate report
    if not args.no_report and not args.update_golden and all_results:
        REPORTS_DIR.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        report_path = REPORTS_DIR / f"report_{timestamp}.html"
        generate_html_report(all_results, report_path)

        latest_path = REPORTS_DIR / "latest.html"
        shutil.copy(report_path, latest_path)
        print(f"\nReport: {report_path}")

    # Summary
    passed = sum(1 for r in all_results if r.passed)
    failed = len(all_results) - passed

    print(f"\n{'=' * 40}")
    if args.update_golden:
        print(f"Updated {passed} golden images")
    elif all_results:
        print(f"Results: {passed} passed, {failed} failed")
    else:
        print("No tests run (all apps skipped)")

    sys.exit(0 if failed == 0 else 1)


if __name__ == "__main__":
    main()
