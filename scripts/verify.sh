#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "=== fmt ==="
cargo fmt --check

echo "=== clippy ==="
cargo clippy -- -D warnings

echo "=== test ==="
cargo test

echo "=== build ==="
cargo build --release

echo ""
echo "ALL CHECKS PASSED"