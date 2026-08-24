# 🦀 Claude Coder Desktop (Tauri v2 in Rust)

[![Tauri v2](https://img.shields.io/badge/Tauri-v2.0-blue.svg?logo=tauri)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![RAM Usage](https://img.shields.io/badge/RAM-%3C60MB-brightgreen.svg)](#-why-tauri--rust-over-electron)

[English 🇬🇧](#english) • [Italiano 🇮🇹](#italiano)

> **The ultra-lightweight, native Rust desktop application for AI-assisted software engineering. Compatible with Claude Code, Ollama, LM Studio, Apple MLX, and 20+ frontier cloud LLM providers.**
> *L'applicazione Desktop nativa ultra-leggera in Rust per lo sviluppo assistito da Intelligenza Artificiale.*

![Claude Coder Desktop Dashboard](./ui/screenshot.jpg)

---

<a name="english"></a>
## 🇬🇧 English Documentation

### ⚡ Why Tauri & Rust over Electron?

* **Near-Zero Memory Footprint**: Uses less than **60 MB of RAM** (compared to 800 MB – 1.5 GB for typical Electron IDEs).
* **Instant Startup**: Launches in under **300 milliseconds**.
* **Native OS Integration**:
  * Native folder pickers on macOS (AppleScript), Windows (FolderBrowserDialog), and Linux (Zenity).
  * 1-Click project dispatch to **Cursor**, **VS Code**, **Windsurf**, or native **Finder / File Explorer**.
  * Rust memory safety and native sandbox security.

### 🌟 Key Capabilities

1. **Complete Developer Console**: Real-time terminal with ANSI streaming, syntax highlighting, and live token economics telemetry.
2. **MCP (Model Context Protocol) Hub**: 11+ preconfigured MCP servers (GitHub, Postgres, SQLite, Puppeteer, Brave Search, Notion, Linear, Slack, Docker, Figma, RuVector) with 1-click export to Cursor and Claude CLI.
3. **Multi-Agent & Auto-Debug Loop**: Built-in `/swarm` multi-agent pipelines, `/diagram` visual Mermaid graphs, `/prd` generator, and `/autofix` test suite self-healing.
4. **Hands-Free Voice-to-Code**: Native voice dictation for prompt input.
5. **Hierarchical Project Memory (Letta/MemGPT)**: 3-tier persistent memory inspector (`.claude/agentdb.json`).

### 🛠️ Build & Installation

```bash
cd claude-coder-tauri
# Run in Development Mode
cargo tauri dev

# Build Production Binary (.dmg / .exe / .AppImage)
cargo tauri build
```

---

<a name="italiano"></a>
## 🇮🇹 Documentazione in Italiano

### ⚡ Perché Tauri in Rust vs Electron?
* **Zero Spreco di RAM**: Occupa meno di **60 MB di RAM** (rispetto agli 800 MB – 1.5 GB delle tipiche app Electron).
* **Avvio Istantaneo**: Tempo di avvio inferiore a **300 millisecondi**.
* **Integrazione Nativa con il Sistema Operativo**:
  * File picker nativo macOS (AppleScript), Windows (FolderBrowserDialog) e Linux (Zenity).
  * Apertura istantanea di file e cartelle in **Cursor**, **VS Code**, **Windsurf** o nel **Finder/File Explorer**.

### 🌟 Funzionalità Chiave
1. **Console di Sviluppo Completa**: Streaming ANSI in tempo reale, evidenziazione sintattica e telemetria token.
2. **Hub Server MCP Integrato**: Marketplace preconfigurato con 11+ server MCP ed esportazione in 1 clic per Cursor e Claude CLI.
3. **Multi-Agente & Auto-Debug**: Loop `/swarm`, diagrammi `/diagram`, specifiche `/prd` e auto-debug `/autofix`.
4. **Dettatura Vocale Hands-Free**: Voice-to-Code nativo per dettare istruzioni vocali direttamente nel prompt.
5. **Memoria Gerarchica (Letta/MemGPT)**: Ispezione e gestione visiva dei 3 livelli di memoria di progetto in `.claude/agentdb.json`.

### 🛠️ Compilazione & Esecuzione
```bash
cd claude-coder-tauri
# Avvio in sviluppo
cargo tauri dev

# Compilazione binario di produzione (.dmg / .exe / .AppImage)
cargo tauri build
```

---

## 📄 License / Licenza
Released under the [MIT License](LICENSE).
