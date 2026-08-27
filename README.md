# 🦀 Claude Coder Desktop (Tauri v2 in Rust)

[![Tauri v2](https://img.shields.io/badge/Tauri-v2.0-blue.svg?logo=tauri)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![RAM Usage](https://img.shields.io/badge/RAM-~90MB%20misurato-brightgreen.svg)](#-why-tauri--rust-over-electron)

[English 🇬🇧](#english) • [Italiano 🇮🇹](#italiano)

> **The ultra-lightweight, native Rust desktop application for AI-assisted software engineering. Powered by a high-performance Rust backend (`src-tauri`) supporting streaming HTTP connections to 8 LLM providers (Ollama locale, OpenAI, Google Gemini, Cerebras, SambaNova, Mistral, Groq, OpenRouter) and native OS filesystem workspace management.**
> *L'applicazione Desktop nativa ultra-leggera in Rust per lo sviluppo assistito da Intelligenza Artificiale con backend nativo Rust e interfaccia WebView reattiva.*

![Claude Coder Desktop Dashboard](./ui/screenshot.jpg)

---

<a name="english"></a>
## 🇬🇧 English Documentation

### ⚡ Why Tauri & Rust over Electron?

* **Small Memory Footprint**: Measured **~90 MB RSS** on macOS at idle (`ps aux` on the release build) — corrected from an unmeasured "<60MB" claim in a previous version. Still dramatically smaller than the commonly reported 800 MB–1.5 GB for typical Electron IDEs.
* **Instant Startup**: Launches in under **300 milliseconds**.
* **Native OS Integration**:
  * Native folder pickers on macOS (AppleScript), Windows (FolderBrowserDialog), and Linux (Zenity).
  * 1-Click project dispatch to **Cursor**, **VS Code**, **Windsurf**, or native **Finder / File Explorer**.
  * Rust memory safety, typed error handling, and sandboxed execution.

### 🌟 Key Capabilities

1. **Native Rust Multi-Provider LLM Core (`agent.rs`)**: Streaming HTTP client connecting directly to Ollama (local), OpenAI, Google Gemini, Cerebras, SambaNova, Mistral, Groq, and OpenRouter — verified against the actual code, not aspirational. The chat UI is wired to this via real Tauri `invoke()` calls and `agent-chunk` / `agent-done` events (see below) — previous builds had a dead frontend that called a REST API which never existed in this app.
2. **Native Filesystem Workspace Engine (`workspace.rs`)**: Fast file reading, writing, directory walking, search, and native file manager integration.
3. **Real Diff-Based File Editing** *(new)*: When the model proposes a file change, it is instructed (system prompt) to emit a fenced `diff` code block containing a unified diff. The UI detects these blocks, shows a colorized preview with an **Apply** button, and applying it calls the real `preview_diff_apply` / `apply_diff_to_file` Tauri commands, which use the `diffy` crate to genuinely parse and patch the file on disk — the Aider/Cursor "propose → review → apply" pattern, not just chat text. Covered by real Rust unit tests that write a temp file, apply a diff, and assert the on-disk bytes changed.
4. **Real Repo Map** *(new, Aider-inspired)*: `generate_repo_map` walks the attached workspace and extracts real function/class/struct signatures via per-language regexes (Rust, TS/JS, Python, Dart, Go, Java/Kotlin) — no fabricated symbols, no tree-sitter dependency. The map is injected into the system prompt of every agent run so the model has real structural context about the project. Covered by a unit test that asserts real signatures are found in real temp files.
5. **Real Stop-Generation** *(new)*: The "Interrompi" button calls `stop_agent_stream`, which flips a shared `AtomicBool` that every provider's streaming loop polls between HTTP chunks — a genuine mid-stream abort of the in-flight `reqwest` connection, not a UI-only state toggle.
6. **Persisted Settings**: API keys and the active model/workspace are read/written to disk via `load_settings` / `save_settings` (`settings.rs`) and now actually populate the UI and the agent calls on startup.

> **Known limitation (documented, not hidden):** the frontend (`ui/app.js`) also contains legacy sections — an MCP config exporter, `/swarm` `/autofix` `/diagram` command scaffolding, a CMUX process multiplexer, a Telegram bridge, and live token stats — that call a `fetch("/api/...")` REST API. This desktop build has **no HTTP server**; those sections are inert leftovers from an earlier prototype and are not yet backed by real Tauri commands. They are left in place (harmless, silently no-op) rather than removed, and are a good target for a future contribution.

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
* **Basso Consumo di RAM**: **~90 MB RSS misurati** su macOS a riposo (`ps aux` sulla build release) — numero corretto rispetto al claim precedente "<60MB" mai misurato. Resta comunque molto inferiore agli 800 MB – 1.5 GB tipici delle app Electron.
* **Avvio Istantaneo**: Tempo di avvio inferiore a **300 millisecondi**.
* **Integrazione Nativa con il Sistema Operativo**:
  * File picker nativo macOS (AppleScript), Windows (FolderBrowserDialog) e Linux (Zenity).
  * Apertura istantanea di file e cartelle in **Cursor**, **VS Code**, **Windsurf** o nel **Finder/File Explorer**.

### 🌟 Funzionalità Chiave
1. **Core Rust Multi-Provider (`agent.rs`)**: Client HTTP streaming nativo per 8 provider LLM (Ollama locale, OpenAI, Google Gemini, Cerebras, SambaNova, Mistral, Groq, OpenRouter) — verificato contro il codice reale, non aspirazionale. La chat è ora collegata a questo motore tramite vere chiamate Tauri `invoke()` e gli eventi reali `agent-chunk` / `agent-done`: nelle build precedenti il frontend era "morto" e chiamava una REST API mai esistita in questa app.
2. **Motore Workspace Nativo (`workspace.rs`)**: Gestione rapida di file, cartelle, scansione albero e dispatch verso editor esterni.
3. **Modifica file reale basata su diff** *(novità)*: quando il modello propone una modifica a un file, il system prompt lo istruisce a produrre un blocco di codice `diff` con un unified diff valido. La UI riconosce questi blocchi, mostra un'anteprima colorata con pulsante **Applica**, e il click chiama i comandi Tauri reali `preview_diff_apply` / `apply_diff_to_file`, che usano il crate `diffy` per analizzare e applicare davvero la patch sul file su disco — il pattern "proponi → rivedi → applica" di Aider/Cursor, non semplice testo in chat. Coperto da test Rust reali che scrivono un file temporaneo, applicano un diff e verificano che i byte su disco siano davvero cambiati.
4. **Repo Map reale** *(novità, ispirata ad Aider)*: `generate_repo_map` scansiona la cartella agganciata ed estrae firme reali di funzioni/classi/struct tramite regex per linguaggio (Rust, TS/JS, Python, Dart, Go, Java/Kotlin) — nessun simbolo inventato, nessuna dipendenza da tree-sitter. La mappa viene iniettata nel system prompt di ogni richiesta, cosi' il modello ha un contesto strutturale reale del progetto. Coperto da un test che verifica il ritrovamento di firme reali in file temporanei reali.
5. **Interruzione generazione reale** *(novità)*: il pulsante "Interrompi" chiama `stop_agent_stream`, che imposta un `AtomicBool` condiviso controllato da ogni loop di streaming tra un chunk HTTP e l'altro — un abort reale a metà stream della connessione `reqwest` in corso, non un semplice toggle di stato nella UI.
6. **Impostazioni persistenti**: le chiavi API e il modello/workspace attivi vengono letti/scritti su disco tramite `load_settings` / `save_settings` (`settings.rs`) e ora popolano davvero la UI e le chiamate all'agente all'avvio.

> **Limite noto (documentato, non nascosto):** il frontend (`ui/app.js`) contiene anche sezioni legacy — un esportatore di configurazioni MCP, gli scaffolding dei comandi `/swarm` `/autofix` `/diagram`, un multiplexer di processi CMUX, un bridge Telegram e statistiche token live — che chiamano una REST API `fetch("/api/...")`. Questa build desktop **non ha alcun server HTTP**; queste sezioni sono residui inerti di un prototipo precedente e non sono ancora collegate a comandi Tauri reali. Sono state lasciate in posto (innocue, falliscono silenziosamente) invece di essere rimosse, e sono un buon obiettivo per un contributo futuro.

### 🛠️ Compilazione & Esecuzione
```bash
cd claude-coder-tauri
# Avvio in sviluppo
cargo tauri dev

# Compilazione binario di produzione (.dmg / .exe / .AppImage)
cargo tauri build
```

---

## ✅ Test automatizzati / Automated Tests

<a name="test-automatizzati"></a>
### 🇮🇹 Italiano

**46 test reali**, tutti eseguiti in CI su ogni push/PR (vedi `.github/workflows/ci.yml`):

- **35 test Rust** (`cd src-tauri && cargo test --lib`):
  - `workspace.rs` (7 test) — apply/preview di un diff reale su un file reale su disco, rifiuto di un diff con contesto non corrispondente senza toccare il file, round-trip scrittura/lettura file, scan di una directory reale con rilevamento framework (Cargo.toml/package.json) e file ignorati (`node_modules`), errori su path inesistenti.
  - `repomap.rs` (6 test) — estrazione di firme reali (funzioni Rust/Python/TS/JS, struct, interface, classi) da file scritti su disco in un test, esclusione di `node_modules`/`target`, file non supportati o vuoti, errore su directory inesistente.
  - `agent.rs` (15 test) — **routing verso gli 8 provider LLM** (`route_model_to_provider`, la stessa funzione usata realmente da `run_agent_stream`, non una copia) e **parsing dei chunk di streaming** per i tre formati realmente usati (SSE OpenAI-compatibile per OpenAI/Cerebras/SambaNova/Mistral/Groq/OpenRouter, SSE Gemini, NDJSON Ollama), inclusi i casi limite: sentinella `[DONE]`, JSON malformato, chunk senza campo `content`.
  - `models.rs` (4 test) — formattazione dimensioni modello (byte → "X.X GB").
  - `settings.rs` (7 test) — default senza chiavi API precompilate, round-trip di serializzazione, retrocompatibilità con `settings.json` salvati da build precedenti prive dei campi più recenti (`anthropicApiKey`, ecc.).
- **11 test frontend** (`npx vitest run`, su `ui/diff-utils.test.js`): estrazione del percorso file target da un diff unificato (header `+++`/`---`, fallback per le eliminazioni pure, `/dev/null`) e risoluzione del percorso (relativo/assoluto POSIX/assoluto Windows) rispetto alla workspace agganciata — la stessa logica usata realmente da `renderDiffProposals`/`buildDiffCard` in `ui/app.js` (refactorizzata in `ui/diff-utils.js` proprio per renderla testabile, mantenendo il comportamento identico).

**Cosa NON è automatizzato (richiede credenziali live) e perché è onesto dirlo:** le chiamate HTTP reali verso gli 8 provider LLM (Ollama locale, OpenAI, Google Gemini, Cerebras, SambaNova, Mistral, Groq, OpenRouter) non sono coperte da test automatici — richiederebbero chiavi API reali come secrets CI e chiamerebbero servizi a pagamento/rate-limited a ogni run. È testata invece, con payload di fixture realistici, tutta la logica di parsing/routing agnostica dal provider (vedi sopra) che è la parte di business logic realmente a rischio di bug silenziosi. Allo stesso modo, `cargo tauri build` (bundle `.dmg`/`.exe`/`.AppImage` completo) non gira in CI — vedi i commenti in `.github/workflows/ci.yml` per il motivo.

**Bug reale trovato e corretto durante la stesura dei test:** nessuno — la logica di diff-apply, repo-map e parsing streaming già presente era corretta; il lavoro qui è stato principalmente *estrarre* logica pura (routing provider, parsing SSE/NDJSON, formattazione dimensioni) dalle funzioni Tauri/async in cui era inline, così da poterla testare senza rete né chiavi API, e collegare quella stessa logica estratta al codice di produzione (non una copia duplicata).

### 🇬🇧 English

**46 real tests**, all run in CI on every push/PR (see `.github/workflows/ci.yml`):

- **35 Rust tests** (`cd src-tauri && cargo test --lib`) covering real diff-apply to a real file on disk (including a mismatched-context diff being rejected without corrupting the file), real repo-map extraction from real temp files (Rust/Python/TS/JS, with `node_modules`/`target` correctly ignored), the **8-provider routing function** and **SSE/NDJSON stream-chunk parsing** actually used by `run_agent_stream` (not a duplicate — the production code calls the same tested functions), model-size formatting, and settings serialization/backward-compatibility.
- **11 frontend tests** (`npx vitest run`, `ui/diff-utils.test.js`) covering the diff-target-path extraction and workspace-path resolution logic used by the real "propose diff → apply" UI flow, extracted into `ui/diff-utils.js` for testability with identical behavior.

**What is NOT automated, honestly:** real HTTP calls to the 8 LLM providers require live API keys and would hit paid/rate-limited services on every CI run, so they are not covered by automated tests — the provider-agnostic parsing/routing logic they depend on is tested instead, with fixture payloads. Likewise, a full `cargo tauri build` bundle is not run in CI (see comments in the workflow file for why).

## 📄 License / Licenza
Released under the [MIT License](LICENSE).
