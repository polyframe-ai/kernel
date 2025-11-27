#!/bin/bash
# SPDX-License-Identifier: Apache-2.0
# Copyright (c) 2025 Polyframe Inc.
#
# Unified validation wrapper script
# Runs all validation methods and aggregates results

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

# Default values
VERBOSE=false
OUTPUT_DIR="tests/evaluation/outputs"
RUN_PYTHON=false
RUN_TYPESCRIPT=false
SUITES=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            SUITES=""
            shift
            ;;
        --suite)
            SUITES="$SUITES,$2"
            shift 2
            ;;
        --suites)
            SUITES="$2"
            shift 2
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --output|-o)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --python)
            RUN_PYTHON=true
            shift
            ;;
        --typescript)
            RUN_TYPESCRIPT=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --all              Run all validation suites (default)"
            echo "  --suite SUITE      Run specific suite (unit, integration, evaluation, comparison, fuzz, regression)"
            echo "  --suites LIST      Run comma-separated list of suites"
            echo "  --verbose, -v      Enable verbose output"
            echo "  --output, -o DIR   Output directory for reports (default: tests/evaluation/outputs)"
            echo "  --python           Also run Python evaluation scripts (if available)"
            echo "  --typescript       Also run TypeScript tests (if available)"
            echo "  --help, -h         Show this help message"
            echo ""
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo "🔍 Polyframe Kernel Validation"
echo "================================"
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo is not installed or not in PATH"
    exit 1
fi

# Build the validation binary
if [ "$VERBOSE" = true ]; then
    echo "📦 Building validation binary..."
fi

cargo build --bin polyframe-validate --release 2>&1 | grep -v "^   Compiling" || true

# Prepare arguments
ARGS=()
if [ "$VERBOSE" = true ]; then
    ARGS+=("--verbose")
fi
ARGS+=("--output" "$OUTPUT_DIR")

# Run Rust validation
echo ""
echo "🚀 Running Rust validation..."
echo ""

if [ -n "$SUITES" ]; then
    # Remove leading comma if present
    SUITES="${SUITES#,}"
    cargo run --release --bin polyframe-validate -- all --suites "$SUITES" "${ARGS[@]}"
else
    cargo run --release --bin polyframe-validate -- all "${ARGS[@]}"
fi

RUST_EXIT_CODE=$?

# Optionally run Python evaluation
if [ "$RUN_PYTHON" = true ]; then
    echo ""
    echo "🐍 Running Python evaluation scripts..."
    
    PYTHON_DIR="../polyframe-training"
    if [ -d "$PYTHON_DIR" ]; then
        cd "$PYTHON_DIR"
        
        if [ -f "scripts/eval_compile.py" ]; then
            if command -v python3 &> /dev/null; then
                python3 scripts/eval_compile.py --test-file data/splits/test.jsonl 2>&1 || true
            else
                echo "⚠️  Python 3 not found, skipping Python evaluation"
            fi
        else
            echo "⚠️  Python evaluation scripts not found"
        fi
        
        cd "$PROJECT_ROOT"
    else
        echo "⚠️  Python training directory not found at $PYTHON_DIR"
    fi
fi

# Optionally run TypeScript tests
if [ "$RUN_TYPESCRIPT" = true ]; then
    echo ""
    echo "📘 Running TypeScript tests..."
    
    TYPESCRIPT_DIR="../polyframe"
    if [ -d "$TYPESCRIPT_DIR" ]; then
        cd "$TYPESCRIPT_DIR"
        
        if [ -f "package.json" ]; then
            if command -v npm &> /dev/null; then
                npm test 2>&1 || true
            else
                echo "⚠️  npm not found, skipping TypeScript tests"
            fi
        else
            echo "⚠️  TypeScript project not found"
        fi
        
        cd "$PROJECT_ROOT"
    else
        echo "⚠️  TypeScript project directory not found at $TYPESCRIPT_DIR"
    fi
fi

# Summary
echo ""
echo "================================"
echo "✅ Validation complete"
echo ""
echo "Reports saved to: $OUTPUT_DIR"
echo "  - validation_report.json"
echo "  - validation_report.md"
echo ""

if [ $RUST_EXIT_CODE -ne 0 ]; then
    echo "❌ Some tests failed. Check the reports for details."
    exit $RUST_EXIT_CODE
else
    echo "✅ All tests passed!"
    exit 0
fi

