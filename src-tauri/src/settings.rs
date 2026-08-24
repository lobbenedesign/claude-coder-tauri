use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    #[serde(rename = "activeModel")]
    pub active_model: String,
    #[serde(rename = "attachedWorkspacePath")]
    pub attached_workspace_path: String,
    #[serde(rename = "geminiApiKey")]
    pub gemini_api_key: String,
    #[serde(rename = "groqApiKey")]
    pub groq_api_key: String,
    #[serde(rename = "openrouterApiKey")]
    pub openrouter_api_key: String,
    #[serde(rename = "cerebrasApiKey")]
    pub cerebras_api_key: String,
    #[serde(rename = "sambanovaApiKey")]
    pub sambanova_api_key: String,
    #[serde(rename = "mistralApiKey")]
    pub mistral_api_key: String,
    #[serde(rename = "openaiApiKey")]
    pub openai_api_key: String,
    #[serde(rename = "anthropicApiKey", default)]
    pub anthropic_api_key: String,
    #[serde(rename = "deepseekApiKey", default)]
    pub deepseek_api_key: String,
    #[serde(rename = "xaiApiKey", default)]
    pub xai_api_key: String,
    #[serde(rename = "togetherApiKey", default)]
    pub together_api_key: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        let default_workspace = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .to_string_lossy()
            .to_string();

        Self {
            active_model: "qwen2.5:7b".to_string(),
            attached_workspace_path: default_workspace,
            gemini_api_key: "".to_string(),
            groq_api_key: "".to_string(),
            openrouter_api_key: "".to_string(),
            cerebras_api_key: "".to_string(),
            sambanova_api_key: "".to_string(),
            mistral_api_key: "".to_string(),
            openai_api_key: "".to_string(),
            anthropic_api_key: "".to_string(),
            deepseek_api_key: "".to_string(),
            xai_api_key: "".to_string(),
            together_api_key: "".to_string(),
        }
    }
}

pub fn get_config_path() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("custom-claude-coder");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir.push("settings.json");
    dir
}

#[tauri::command]
pub fn load_settings() -> AppSettings {
    let path = get_config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                return settings;
            }
        }
    }
    AppSettings::default()
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<bool, String> {
    let path = get_config_path();
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(true)
}
