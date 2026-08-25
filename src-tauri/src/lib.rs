pub mod agent;
pub mod models;
pub mod repomap;
pub mod settings;
pub mod workspace;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri_plugin_dialog::init as init_dialog;
use tauri_plugin_fs::init as init_fs;
use tauri_plugin_opener::init as init_opener;
use tauri_plugin_shell::init as init_shell;

/// Shared cancellation flag for the currently in-flight agent stream. Set to
/// true by `stop_agent_stream` to abort the real HTTP stream mid-flight
/// (checked between chunks in agent.rs) — a genuine stop, not just a UI
/// state toggle.
pub struct AgentState {
    pub cancel_flag: Arc<AtomicBool>,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(init_dialog())
        .plugin(init_fs())
        .plugin(init_shell())
        .plugin(init_opener())
        .manage(AgentState::default())
        .invoke_handler(tauri::generate_handler![
            settings::load_settings,
            settings::save_settings,
            workspace::scan_workspace,
            workspace::read_workspace_file,
            workspace::write_workspace_file,
            workspace::open_in_editor,
            workspace::pick_folder,
            workspace::preview_diff_apply,
            workspace::apply_diff_to_file,
            models::get_installed_ollama_models,
            agent::run_agent_stream,
            agent::stop_agent_stream,
            repomap::generate_repo_map,
        ])
        .run(tauri::generate_context!())
        .expect("Errore durante l'avvio dell'applicazione Tauri");
}
