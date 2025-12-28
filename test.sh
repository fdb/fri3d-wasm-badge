#!/bin/bash
# Run tests for Fri3d Badge App
# Usage: ./test.sh
#
# This runs all available tests.

set -e

echo "Running visual tests..."
uv run tests/visual/run_visual_tests.py

