#!/bin/bash
# Quick test script for parser development
# Usage: ./test-parser.sh [test-name]

set -e

cd "$(dirname "$0")"

echo "🧪 Testing Polyframe Kernel Parser"
echo "=================================="
echo ""

if [ -z "$1" ]; then
    echo "Running all parser tests..."
    cargo test --lib io::parser::tests -- --nocapture
else
    echo "Running specific test: $1"
    cargo test --lib "$1" -- --nocapture
fi

echo ""
echo "✅ Tests completed!"

