pub mod agent;
pub mod models;
pub mod settings;
pub mod workspace;

use tauri_plugin_dialog::init as init_dialog;
use tauri_plugin_fs::init as init_fs;
use tauri_plugin_opener::init as init_opener;
use tauri_plugin_shell::init as init_shell;

pub fn run() {
    tauri::Builder::default()
        .plugin(init_dialog())
        .plugin(init_fs())
        .plugin(init_shell())
        .plugin(init_opener())
        .invoke_handler(tauri::generate_handler![
            settings::load_settings,
            settings::save_settings,
            workspace::scan_workspace,
            workspace::read_workspace_file,
            workspace::write_workspace_file,
            workspace::open_in_editor,
            workspace::pick_folder,
            models::get_installed_ollama_models,
            agent::run_agent_stream,
        ])
        .run(tauri::generate_context!())
        .expect("Errore durante l'avvio dell'applicazione Tauri");
}
