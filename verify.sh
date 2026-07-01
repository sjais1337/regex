#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

echo "building..."
cargo build --quiet

echo "running unit tests..."
cargo test --quiet

echo "running smoke checks..."
cargo run --quiet 2>/dev/null | grep -q 'true'

echo "all verification passed"
