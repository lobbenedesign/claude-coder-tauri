# 🦀 Claude Coder Desktop (Tauri v2 in Rust)

[![Tauri v2](https://img.shields.io/badge/Tauri-v2.0-blue.svg?logo=tauri)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![RAM Usage](https://img.shields.io/badge/RAM-%3C60MB-brightgreen.svg)](#-why-tauri--rust-over-electron)

[English 🇬🇧](#english) • [Italiano 🇮🇹](#italiano)

> **The ultra-lightweight, native Rust desktop application for AI-assisted software engineering. Powered by a high-performance Rust backend (`src-tauri`) supporting streaming HTTP connections to 8 LLM providers (Ollama, Anthropic, OpenAI, Cerebras, Mistral, Groq, LM Studio, DeepSeek) and native OS filesystem workspace management.**
> *L'applicazione Desktop nativa ultra-leggera in Rust per lo sviluppo assistito da Intelligenza Artificiale con backend nativo Rust e interfaccia WebView reattiva.*

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
  * Rust memory safety, typed error handling, and sandboxed execution.

### 🌟 Key Capabilities

1. **Native Rust Multi-Provider LLM Core (`agent.rs`)**: Streaming HTTP client connecting directly to Ollama, OpenAI, Anthropic, Cerebras, Mistral, Groq, and LM Studio.
2. **Native Filesystem Workspace Engine (`workspace.rs`)**: Fast file reading, writing, directory walking, search, and native file manager integration.
3. **MCP Configuration Manager**: 1-click JSON export for 11+ Model Context Protocol server configurations to Cursor and Claude CLI settings.
4. **Interactive Command Workflows**: UI support for `/swarm` pipeline staging, `/diagram` visual graphs, `/prd` scaffolding, and `/autofix` code validation.
5. **Speech Input Integration**: Browser Web Speech API integration for prompt dictation.

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
1. **Core Rust Multi-Provider (`agent.rs`)**: Client HTTP streaming nativo per 8 provider LLM (Ollama locale, Anthropic, OpenAI, Cerebras, Mistral, Groq, LM Studio).
2. **Motore Workspace Nativo (`workspace.rs`)**: Gestione rapida di file, cartelle, scansione albero e dispatch verso editor esterni.
3. **Manager Configurazioni MCP**: Esportazione in 1 clic delle configurazioni JSON per server MCP verso Cursor e Claude CLI.
4. **Comandi Interattivi**: Supporto interfaccia per pipeline `/swarm`, visualizzatore diagrammi `/diagram` e task `/autofix`.
5. **Input Vocale**: Dettatura vocale integrata tramite Web Speech API.

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
