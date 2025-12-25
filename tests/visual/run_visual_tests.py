#!/usr/bin/env python3
"""
Visual Regression Testing for FRI3D WASM Badge

This script captures screenshots from the test_drawing app and compares them
against golden (reference) screenshots. It generates a detailed HTML report
showing expected, actual, and diff images.

Usage:
    python run_visual_tests.py                    # Run all tests
    python run_visual_tests.py --update-golden    # Update golden screenshots
    python run_visual_tests.py --scene 0          # Run single scene test
"""

import argparse
import os
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from typing import Optional, Tuple, List, NamedTuple

try:
    from PIL import Image
except ImportError:
    print("Error: Pillow is required. Install with: pip install Pillow")
    sys.exit(1)


# Scene definitions matching test_drawing/main.c
SCENES = [
    ("horizontal_lines", "Horizontal Lines"),
    ("vertical_lines", "Vertical Lines"),
    ("diagonal_lines", "Diagonal Lines"),
    ("random_pixels", "Random Pixels"),
    ("circles", "Circles"),
    ("filled_circles", "Filled Circles"),
    ("rectangles", "Rectangles"),
    ("filled_rectangles", "Filled Rectangles"),
    ("rounded_rectangles", "Rounded Rectangles"),
    ("text_rendering", "Text Rendering"),
    ("mixed_primitives", "Mixed Primitives"),
    ("checkerboard", "Checkerboard"),
]

# Paths
SCRIPT_DIR = Path(__file__).parent.absolute()
PROJECT_ROOT = SCRIPT_DIR.parent.parent
HOST_EMULATOR = PROJECT_ROOT / "build" / "src" / "host" / "host_emulator"
GOLDEN_DIR = SCRIPT_DIR / "golden"
ACTUAL_DIR = SCRIPT_DIR / "actual"
DIFF_DIR = SCRIPT_DIR / "diff"
REPORTS_DIR = SCRIPT_DIR / "reports"


def find_test_app() -> Optional[Path]:
    """Find the test_drawing WASM app, checking multiple possible locations."""
    candidates = [
        PROJECT_ROOT / "build-apps" / "test_drawing" / "test_drawing.wasm",
        PROJECT_ROOT / "build-apps" / "test_drawing" / "test_drawing",
    ]

    # Also search for any .wasm file in the build directory
    build_dir = PROJECT_ROOT / "build-apps" / "test_drawing"
    if build_dir.exists():
        for f in build_dir.glob("*.wasm"):
            if f not in candidates:
                candidates.append(f)
        # Check for file without extension (Zig might not add .wasm)
        for f in build_dir.iterdir():
            if f.is_file() and f.suffix == "" and f.name == "test_drawing":
                if f not in candidates:
                    candidates.append(f)

    for path in candidates:
        if path.exists():
            return path
    return None


TEST_APP: Optional[Path] = None  # Will be set by find_test_app()


class TestResult(NamedTuple):
    """Result of a single visual test."""
    scene_id: int
    scene_name: str
    scene_title: str
    passed: bool
    diff_count: int
    golden_path: Optional[Path]
    actual_path: Optional[Path]
    diff_path: Optional[Path]
    error: Optional[str]


def ensure_directories():
    """Create required directories if they don't exist."""
    for dir_path in [GOLDEN_DIR, ACTUAL_DIR, DIFF_DIR, REPORTS_DIR]:
        dir_path.mkdir(parents=True, exist_ok=True)


def check_prerequisites() -> bool:
    """Check if host emulator and test app exist."""
    global TEST_APP

    if not HOST_EMULATOR.exists():
        print(f"Error: Host emulator not found at {HOST_EMULATOR}")
        print("Run 'cmake -B build && cmake --build build' to build the host emulator.")
        return False

    TEST_APP = find_test_app()
    if TEST_APP is None:
        build_dir = PROJECT_ROOT / "build-apps" / "test_drawing"
        print(f"Error: Test app not found in {build_dir}")
        if build_dir.exists():
            print(f"Files in build directory: {list(build_dir.iterdir())}")
        print("Run the build script to compile the test_drawing app:")
        print(f"  cmake -B build-apps/test_drawing -DCMAKE_TOOLCHAIN_FILE=cmake/toolchain-wasm.cmake -S src/apps/test_drawing")
        print(f"  cmake --build build-apps/test_drawing")
        return False

    print(f"Found test app: {TEST_APP}")
    return True


def capture_screenshot(scene_id: int, output_path: Path) -> bool:
    """Capture a screenshot for a specific scene."""
    cmd = [
        str(HOST_EMULATOR),
        "--headless",
        "--scene", str(scene_id),
        "--screenshot", str(output_path),
        str(TEST_APP)
    ]

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        if result.returncode != 0:
            print(f"Error capturing screenshot: {result.stderr}")
            return False
        return output_path.exists()
    except subprocess.TimeoutExpired:
        print(f"Timeout capturing screenshot for scene {scene_id}")
        return False
    except Exception as e:
        print(f"Exception capturing screenshot: {e}")
        return False


def load_image(path: Path) -> Optional[Image.Image]:
    """Load an image file and convert to grayscale."""
    if not path.exists():
        return None
    try:
        img = Image.open(path)
        return img.convert('L')  # Convert to grayscale
    except Exception as e:
        print(f"Error loading image {path}: {e}")
        return None


def compare_images(golden: Image.Image, actual: Image.Image) -> Tuple[int, Image.Image]:
    """
    Compare two images pixel by pixel.

    Returns:
        Tuple of (diff_count, diff_image)
        - diff_count: number of differing pixels
        - diff_image: RGB image showing differences
          - Green: pixels present in actual but not in golden (added)
          - Red: pixels present in golden but not in actual (removed)
          - Gray: unchanged pixels
    """
    if golden.size != actual.size:
        raise ValueError(f"Image size mismatch: golden={golden.size}, actual={actual.size}")

    width, height = golden.size
    diff_img = Image.new('RGB', (width, height))
    diff_pixels = diff_img.load()
    golden_pixels = golden.load()
    actual_pixels = actual.load()

    diff_count = 0

    for y in range(height):
        for x in range(width):
            g_pixel = golden_pixels[x, y]
            a_pixel = actual_pixels[x, y]

            # Threshold for "black" (drawn pixel)
            g_black = g_pixel < 128
            a_black = a_pixel < 128

            if g_black == a_black:
                # Same - show as gray (lighter for white, darker for black)
                gray_val = 50 if g_black else 200
                diff_pixels[x, y] = (gray_val, gray_val, gray_val)
            elif a_black and not g_black:
                # Added pixel (in actual but not in golden) - GREEN
                diff_pixels[x, y] = (0, 255, 0)
                diff_count += 1
            else:
                # Removed pixel (in golden but not in actual) - RED
                diff_pixels[x, y] = (255, 0, 0)
                diff_count += 1

    return diff_count, diff_img


def run_test(scene_id: int, scene_name: str, scene_title: str) -> TestResult:
    """Run a visual test for a single scene."""
    golden_path = GOLDEN_DIR / f"{scene_name}.png"
    actual_path = ACTUAL_DIR / f"{scene_name}.png"
    diff_path = DIFF_DIR / f"{scene_name}_diff.png"

    # Capture screenshot
    if not capture_screenshot(scene_id, actual_path):
        return TestResult(
            scene_id=scene_id,
            scene_name=scene_name,
            scene_title=scene_title,
            passed=False,
            diff_count=-1,
            golden_path=None,
            actual_path=None,
            diff_path=None,
            error="Failed to capture screenshot"
        )

    # Check if golden exists
    if not golden_path.exists():
        return TestResult(
            scene_id=scene_id,
            scene_name=scene_name,
            scene_title=scene_title,
            passed=False,
            diff_count=-1,
            golden_path=None,
            actual_path=actual_path,
            diff_path=None,
            error="Golden screenshot not found. Run with --update-golden to create."
        )

    # Load images
    golden_img = load_image(golden_path)
    actual_img = load_image(actual_path)

    if golden_img is None or actual_img is None:
        return TestResult(
            scene_id=scene_id,
            scene_name=scene_name,
            scene_title=scene_title,
            passed=False,
            diff_count=-1,
            golden_path=golden_path,
            actual_path=actual_path,
            diff_path=None,
            error="Failed to load images"
        )

    # Compare images
    try:
        diff_count, diff_img = compare_images(golden_img, actual_img)
        diff_img.save(diff_path)
    except Exception as e:
        return TestResult(
            scene_id=scene_id,
            scene_name=scene_name,
            scene_title=scene_title,
            passed=False,
            diff_count=-1,
            golden_path=golden_path,
            actual_path=actual_path,
            diff_path=None,
            error=f"Comparison failed: {e}"
        )

    return TestResult(
        scene_id=scene_id,
        scene_name=scene_name,
        scene_title=scene_title,
        passed=(diff_count == 0),
        diff_count=diff_count,
        golden_path=golden_path,
        actual_path=actual_path,
        diff_path=diff_path,
        error=None
    )


def update_golden_screenshots(scene_ids: Optional[List[int]] = None):
    """Update golden screenshots by capturing current output."""
    if scene_ids is None:
        scene_ids = list(range(len(SCENES)))

    print("Updating golden screenshots...")
    for scene_id in scene_ids:
        scene_name, scene_title = SCENES[scene_id]
        golden_path = GOLDEN_DIR / f"{scene_name}.png"

        print(f"  Scene {scene_id}: {scene_title}...", end=" ")
        if capture_screenshot(scene_id, golden_path):
            print("OK")
        else:
            print("FAILED")


def generate_html_report(results: List[TestResult], output_path: Path):
    """Generate an HTML report showing test results."""
    passed = sum(1 for r in results if r.passed)
    failed = len(results) - passed

    # Convert images to base64 for embedding in HTML
    def img_to_data_uri(path: Optional[Path]) -> str:
        if path is None or not path.exists():
            return ""

        import base64

        # Convert PPM to PNG for better HTML compatibility
        try:
            img = Image.open(path)
            # Scale up 4x for better visibility
            img = img.resize((img.width * 4, img.height * 4), Image.NEAREST)

            from io import BytesIO
            buffer = BytesIO()
            img.save(buffer, format='PNG')
            b64 = base64.b64encode(buffer.getvalue()).decode()
            return f"data:image/png;base64,{b64}"
        except Exception:
            return ""

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Visual Regression Test Report</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        h1 {{
            color: #333;
            border-bottom: 2px solid #333;
            padding-bottom: 10px;
        }}
        .summary {{
            background: white;
            padding: 20px;
            border-radius: 8px;
            margin-bottom: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .summary-stats {{
            display: flex;
            gap: 20px;
            margin-top: 10px;
        }}
        .stat {{
            padding: 10px 20px;
            border-radius: 4px;
            font-weight: bold;
        }}
        .stat.passed {{
            background: #d4edda;
            color: #155724;
        }}
        .stat.failed {{
            background: #f8d7da;
            color: #721c24;
        }}
        .test-result {{
            background: white;
            padding: 20px;
            border-radius: 8px;
            margin-bottom: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .test-result.passed {{
            border-left: 4px solid #28a745;
        }}
        .test-result.failed {{
            border-left: 4px solid #dc3545;
        }}
        .test-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 15px;
        }}
        .test-title {{
            font-size: 1.2em;
            font-weight: bold;
            color: #333;
        }}
        .test-status {{
            padding: 5px 15px;
            border-radius: 20px;
            font-weight: bold;
            font-size: 0.9em;
        }}
        .test-status.passed {{
            background: #28a745;
            color: white;
        }}
        .test-status.failed {{
            background: #dc3545;
            color: white;
        }}
        .images {{
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 20px;
        }}
        .image-container {{
            text-align: center;
        }}
        .image-container h4 {{
            margin: 0 0 10px 0;
            color: #666;
        }}
        .image-container img {{
            border: 1px solid #ddd;
            background: white;
            image-rendering: pixelated;
            max-width: 100%;
        }}
        .error {{
            background: #fff3cd;
            color: #856404;
            padding: 10px;
            border-radius: 4px;
            margin-top: 10px;
        }}
        .legend {{
            background: white;
            padding: 15px;
            border-radius: 8px;
            margin-bottom: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .legend h3 {{
            margin-top: 0;
        }}
        .legend-item {{
            display: inline-flex;
            align-items: center;
            margin-right: 20px;
        }}
        .legend-color {{
            width: 20px;
            height: 20px;
            margin-right: 8px;
            border: 1px solid #ccc;
        }}
        .legend-color.green {{
            background: #00ff00;
        }}
        .legend-color.red {{
            background: #ff0000;
        }}
        .legend-color.gray-dark {{
            background: #323232;
        }}
        .legend-color.gray-light {{
            background: #c8c8c8;
        }}
        .diff-count {{
            font-size: 0.9em;
            color: #666;
            margin-left: 10px;
        }}
    </style>
</head>
<body>
    <h1>Visual Regression Test Report</h1>

    <div class="summary">
        <strong>Generated:</strong> {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
        <div class="summary-stats">
            <div class="stat passed">{passed} Passed</div>
            <div class="stat failed">{failed} Failed</div>
        </div>
    </div>

    <div class="legend">
        <h3>Diff Legend</h3>
        <div class="legend-item">
            <div class="legend-color green"></div>
            <span>Added pixels (in actual, not in golden)</span>
        </div>
        <div class="legend-item">
            <div class="legend-color red"></div>
            <span>Removed pixels (in golden, not in actual)</span>
        </div>
        <div class="legend-item">
            <div class="legend-color gray-dark"></div>
            <span>Unchanged black pixels</span>
        </div>
        <div class="legend-item">
            <div class="legend-color gray-light"></div>
            <span>Unchanged white pixels</span>
        </div>
    </div>
"""

    for result in results:
        status_class = "passed" if result.passed else "failed"
        status_text = "PASSED" if result.passed else "FAILED"

        html += f"""
    <div class="test-result {status_class}">
        <div class="test-header">
            <span class="test-title">Scene {result.scene_id}: {result.scene_title}</span>
            <span>
                <span class="test-status {status_class}">{status_text}</span>
                {"<span class='diff-count'>" + str(result.diff_count) + " pixels differ</span>" if result.diff_count > 0 else ""}
            </span>
        </div>
"""

        if result.error:
            html += f"""
        <div class="error">{result.error}</div>
"""
        else:
            golden_uri = img_to_data_uri(result.golden_path)
            actual_uri = img_to_data_uri(result.actual_path)
            diff_uri = img_to_data_uri(result.diff_path)

            html += f"""
        <div class="images">
            <div class="image-container">
                <h4>Expected (Golden)</h4>
                {"<img src='" + golden_uri + "' alt='Golden'>" if golden_uri else "<p>Not available</p>"}
            </div>
            <div class="image-container">
                <h4>Actual</h4>
                {"<img src='" + actual_uri + "' alt='Actual'>" if actual_uri else "<p>Not available</p>"}
            </div>
            <div class="image-container">
                <h4>Diff</h4>
                {"<img src='" + diff_uri + "' alt='Diff'>" if diff_uri else "<p>Not available</p>"}
            </div>
        </div>
"""

        html += """
    </div>
"""

    html += """
</body>
</html>
"""

    output_path.write_text(html)
    print(f"\nReport generated: {output_path}")


def has_golden_screenshots() -> bool:
    """Check if golden screenshots exist."""
    if not GOLDEN_DIR.exists():
        return False
    png_files = list(GOLDEN_DIR.glob("*.png"))
    return len(png_files) > 0


def main():
    parser = argparse.ArgumentParser(description="Visual regression testing for FRI3D WASM Badge")
    parser.add_argument("--update-golden", action="store_true",
                        help="Update golden screenshots with current output")
    parser.add_argument("--scene", type=int, metavar="N",
                        help="Run test for specific scene only")
    parser.add_argument("--no-report", action="store_true",
                        help="Skip HTML report generation")
    args = parser.parse_args()

    ensure_directories()

    if not check_prerequisites():
        sys.exit(1)

    # Determine which scenes to test
    if args.scene is not None:
        if args.scene < 0 or args.scene >= len(SCENES):
            print(f"Error: Invalid scene number. Valid range: 0-{len(SCENES)-1}")
            sys.exit(1)
        scene_ids = [args.scene]
    else:
        scene_ids = list(range(len(SCENES)))

    # Update golden screenshots if requested
    if args.update_golden:
        update_golden_screenshots(scene_ids)
        print("\nGolden screenshots updated successfully!")
        return

    # Check if golden screenshots exist
    if not has_golden_screenshots():
        print("=" * 60)
        print("WARNING: No golden screenshots found!")
        print("=" * 60)
        print()
        print("Visual regression tests cannot run without reference images.")
        print("To generate golden screenshots, run:")
        print()
        print("    python scripts/update_golden_screenshots.py")
        print()
        print("Then commit the generated files in tests/visual/golden/")
        print()
        print("Skipping tests (this is not a failure).")
        sys.exit(0)

    # Run tests
    print("Running visual regression tests...\n")
    results = []

    for scene_id in scene_ids:
        scene_name, scene_title = SCENES[scene_id]
        print(f"Testing scene {scene_id}: {scene_title}...", end=" ")

        result = run_test(scene_id, scene_name, scene_title)
        results.append(result)

        if result.passed:
            print("PASSED")
        elif result.error:
            print(f"FAILED - {result.error}")
        else:
            print(f"FAILED - {result.diff_count} pixels differ")

    # Generate report
    if not args.no_report:
        report_path = REPORTS_DIR / f"report_{datetime.now().strftime('%Y%m%d_%H%M%S')}.html"
        generate_html_report(results, report_path)

        # Also create a "latest" symlink/copy
        latest_path = REPORTS_DIR / "latest.html"
        if latest_path.exists():
            latest_path.unlink()
        # Use copy instead of symlink for better compatibility
        import shutil
        shutil.copy(report_path, latest_path)

    # Print summary
    passed = sum(1 for r in results if r.passed)
    failed = len(results) - passed
    print(f"\n{'='*50}")
    print(f"Results: {passed} passed, {failed} failed")

    # Exit with error code if any tests failed
    sys.exit(0 if failed == 0 else 1)


if __name__ == "__main__":
    main()
