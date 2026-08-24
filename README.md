# 🦀 Claude Coder Desktop (Tauri v2 in Rust)

[![Tauri v2](https://img.shields.io/badge/Tauri-v2.0-blue.svg?logo=tauri)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![RAM Usage](https://img.shields.io/badge/RAM-%3C60MB-brightgreen.svg)](#-why-tauri-rust-over-electron)

> **The ultra-lightweight, native Rust desktop application for AI-assisted software engineering. Compatible with Claude Code, Ollama, LM Studio, Apple MLX, and 20+ frontier cloud LLM providers.**

![Claude Coder Desktop Dashboard](./ui/screenshot.jpg)

---

## ⚡ Why Tauri & Rust over Electron?

* **Near-Zero Memory Footprint**: Uses less than **60 MB of RAM** (compared to 800 MB – 1.5 GB for typical Electron IDEs).
* **Instant Startup**: Launches in under **300 milliseconds**.
* **Native OS Integration**:
  * Native folder pickers on macOS (AppleScript), Windows (FolderBrowserDialog), and Linux (Zenity).
  * 1-Click project dispatch to **Cursor**, **VS Code**, **Windsurf**, or native **Finder / File Explorer**.
  * Rust memory safety and native sandbox security.

---

## 🌟 Key Capabilities

1. **Complete Developer Console**: Real-time terminal with ANSI streaming, syntax highlighting, and live token economics telemetry.
2. **MCP (Model Context Protocol) Hub**: 11+ preconfigured MCP servers (GitHub, Postgres, SQLite, Puppeteer, Brave Search, Notion, Linear, Slack, Docker, Figma, RuVector) with 1-click export to Cursor and Claude CLI.
3. **Multi-Agent & Auto-Debug Loop**: Built-in `/swarm` multi-agent pipelines, `/diagram` visual Mermaid graphs, `/prd` generator, and `/autofix` test suite self-healing.
4. **Hands-Free Voice-to-Code**: Native voice dictation for prompt input.
5. **Hierarchical Project Memory (Letta/MemGPT)**: 3-tier persistent memory inspector (`.claude/agentdb.json`).

---

## 🛠️ Build & Installation

### Prerequisites
* **Rust**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
* **Node.js / Bun**: For building the frontend UI.

### Run in Development Mode
```bash
cd claude-coder-tauri
cargo tauri dev
```

### Build Production Bundle (.dmg / .exe / .AppImage)
```bash
cargo tauri build
```

The optimized release binary is generated in `src-tauri/target/release/bundle/`.

---

## 📄 License
Released under the [MIT License](LICENSE).
