#!/bin/bash
cd "$(dirname "$0")/src-tauri" || exit 1
echo "======================================================"
echo "🚀 Avvio CUSTOM CLAUDE CODER (Tauri 2.0 Rust Desktop)"
echo "======================================================"
cargo run
