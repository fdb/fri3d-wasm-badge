#!/usr/bin/env python3
"""
Trace-Based Integration Tests for FRI3D WASM Badge

Usage:
    python3 tests/trace/run_trace_tests.py           # Run all specs
    python3 tests/trace/run_trace_tests.py --app circles
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

SCRIPT_DIR = Path(__file__).parent.absolute()
PROJECT_ROOT = SCRIPT_DIR.parent.parent
SPECS_DIR = SCRIPT_DIR / "specs"
EXPECTED_DIR = SCRIPT_DIR / "expected"
OUTPUT_DIR = SCRIPT_DIR / "output"
TRACE_HARNESS = PROJECT_ROOT / "build" / "trace" / "bin" / "trace_harness"


@dataclass
class TestResult:
    spec_path: Path
    passed: bool
    error: str | None = None


def find_wasm_binary(app_id: str) -> Path | None:
    build_dir = PROJECT_ROOT / "build" / "apps" / app_id
    if not build_dir.exists():
        return None

    for pattern in [f"{app_id}.wasm", app_id]:
        candidate = build_dir / pattern
        if candidate.exists():
            return candidate

    wasm_files = list(build_dir.glob("*.wasm"))
    return wasm_files[0] if wasm_files else None


def discover_specs() -> list[Path]:
    if not SPECS_DIR.exists():
        return []
    return sorted(path for path in SPECS_DIR.rglob("*.json") if path.is_file())


def load_json(path: Path) -> Any:
    with open(path) as f:
        return json.load(f)


def event_matches(actual: dict[str, Any], expected: dict[str, Any]) -> bool:
    for key in ("frame", "fn", "ret"):
        if key in expected and actual.get(key) != expected[key]:
            return False

    if "args" in expected:
        actual_args = actual.get("args")
        if not isinstance(actual_args, list):
            return False
        if len(actual_args) != len(expected["args"]):
            return False
        for idx, value in enumerate(expected["args"]):
            if actual_args[idx] != value:
                return False

    return True


def contains_required(trace_events: list[dict[str, Any]],
                      required: list[dict[str, Any]],
                      ordered: bool) -> tuple[bool, str | None]:
    if not required:
        return True, None

    if ordered:
        idx = 0
        for entry in required:
            match_idx = None
            for i in range(idx, len(trace_events)):
                if event_matches(trace_events[i], entry):
                    match_idx = i
                    break
            if match_idx is None:
                return False, f"Missing required event: {entry}"
            idx = match_idx + 1
        return True, None

    used = [False] * len(trace_events)
    for entry in required:
        found = False
        for i, event in enumerate(trace_events):
            if used[i]:
                continue
            if event_matches(event, entry):
                used[i] = True
                found = True
                break
        if not found:
            return False, f"Missing required event: {entry}"
    return True, None


def check_min_calls(trace_events: list[dict[str, Any]],
                    min_calls: list[dict[str, Any]]) -> tuple[bool, str | None]:
    for rule in min_calls:
        fn = rule.get("fn")
        minimum = rule.get("min")
        if not fn or minimum is None:
            continue
        count = sum(1 for event in trace_events if event.get("fn") == fn)
        if count < minimum:
            return False, f"Expected at least {minimum} calls to {fn}, got {count}"
    return True, None


def run_spec(spec_path: Path) -> TestResult:
    try:
        spec = load_json(spec_path)
    except Exception as exc:
        return TestResult(spec_path, False, f"Failed to load spec: {exc}")

    app_id = spec.get("app") or spec_path.parent.name
    frames = int(spec.get("frames", 1))
    seed = int(spec.get("seed", 42))
    frame_ms = int(spec.get("frame_ms", 16))
    mode = spec.get("mode", "fixed")
    duration_ms = spec.get("duration_ms")
    scene = spec.get("scene")
    expected_mode = spec.get("expected_mode", "contains")
    required = spec.get("required", [])
    ordered = bool(spec.get("ordered", True))
    min_calls = spec.get("min_calls", [])

    wasm_path = find_wasm_binary(app_id)
    if not wasm_path:
        return TestResult(spec_path, False, f"WASM binary not found for {app_id}")

    if not TRACE_HARNESS.exists():
        return TestResult(
            spec_path,
            False,
            f"Trace harness not found at {TRACE_HARNESS}. Build with: cmake -B build/trace -DBUILD_TRACE_TESTS=ON",
        )

    output_dir = OUTPUT_DIR / app_id
    output_dir.mkdir(parents=True, exist_ok=True)
    trace_path = output_dir / f"{spec_path.stem}.json"

    input_path = None
    if "input_file" in spec:
        input_path = (spec_path.parent / spec["input_file"]).resolve()
    elif "input" in spec:
        input_path = output_dir / f"{spec_path.stem}_input.json"
        with open(input_path, "w") as f:
            json.dump(spec["input"], f, indent=2)

    cmd = [
        str(TRACE_HARNESS),
        "--app", str(wasm_path),
        "--out", str(trace_path),
        "--frames", str(frames),
        "--seed", str(seed),
        "--frame-ms", str(frame_ms),
        "--app-id", str(app_id),
    ]
    if mode:
        cmd += ["--mode", str(mode)]
    if duration_ms is not None:
        cmd += ["--duration-ms", str(duration_ms)]
    if input_path:
        cmd += ["--input", str(input_path)]
    if scene is not None:
        cmd += ["--scene", str(scene)]

    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        error = result.stderr.strip() or result.stdout.strip() or "Trace harness failed"
        return TestResult(spec_path, False, error)

    if not trace_path.exists():
        return TestResult(spec_path, False, "Trace output missing")

    try:
        trace = load_json(trace_path)
    except Exception as exc:
        return TestResult(spec_path, False, f"Failed to load trace: {exc}")

    trace_events = trace.get("events", [])

    if expected_mode == "exact":
        expected_path = EXPECTED_DIR / app_id / f"{spec_path.stem}.json"
        if not expected_path.exists():
            return TestResult(spec_path, False, f"Expected trace not found: {expected_path}")
        try:
            expected_trace = load_json(expected_path)
        except Exception as exc:
            return TestResult(spec_path, False, f"Failed to load expected trace: {exc}")
        if expected_trace != trace:
            return TestResult(spec_path, False, "Trace does not match expected output")
        return TestResult(spec_path, True)

    ok, message = contains_required(trace_events, required, ordered)
    if not ok:
        return TestResult(spec_path, False, message)

    ok, message = check_min_calls(trace_events, min_calls)
    if not ok:
        return TestResult(spec_path, False, message)

    return TestResult(spec_path, True)


def main() -> int:
    parser = argparse.ArgumentParser(description="Run trace-based integration tests")
    parser.add_argument("--app", help="Run tests only for a specific app id")
    args = parser.parse_args()

    specs = discover_specs()
    if args.app:
        specs = [spec for spec in specs if spec.parent.name == args.app or load_json(spec).get("app") == args.app]

    if not specs:
        print("No trace specs found.")
        return 1

    results: list[TestResult] = []
    for spec in specs:
        print(f"Running {spec.relative_to(PROJECT_ROOT)}...")
        result = run_spec(spec)
        results.append(result)
        if result.passed:
            print("  PASS")
        else:
            print(f"  FAIL: {result.error}")

    failed = [r for r in results if not r.passed]
    print("")
    print(f"Trace tests: {len(results) - len(failed)} passed, {len(failed)} failed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
