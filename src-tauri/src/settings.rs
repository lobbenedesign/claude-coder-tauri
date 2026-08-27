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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_have_no_api_keys_prefilled() {
        // Every API key must default to empty — a non-empty default would be a
        // real security/UX bug (a key nobody typed being sent to a provider).
        let s = AppSettings::default();
        assert_eq!(s.gemini_api_key, "");
        assert_eq!(s.groq_api_key, "");
        assert_eq!(s.openrouter_api_key, "");
        assert_eq!(s.cerebras_api_key, "");
        assert_eq!(s.sambanova_api_key, "");
        assert_eq!(s.mistral_api_key, "");
        assert_eq!(s.openai_api_key, "");
        assert_eq!(s.anthropic_api_key, "");
        assert_eq!(s.deepseek_api_key, "");
        assert_eq!(s.xai_api_key, "");
        assert_eq!(s.together_api_key, "");
    }

    #[test]
    fn default_settings_have_a_non_empty_workspace_and_model() {
        let s = AppSettings::default();
        assert!(!s.attached_workspace_path.is_empty());
        assert_eq!(s.active_model, "qwen2.5:7b");
    }

    #[test]
    fn settings_survive_a_json_serialize_deserialize_round_trip() {
        // Simulates the real load_settings/save_settings path (which is just
        // serde_json::to_string_pretty + fs::write, and fs::read_to_string +
        // serde_json::from_str) without touching the real user config dir.
        let mut original = AppSettings::default();
        original.active_model = "gpt-4o".to_string();
        original.openai_api_key = "sk-test-not-a-real-key".to_string();

        let json = serde_json::to_string_pretty(&original).unwrap();
        let restored: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.active_model, "gpt-4o");
        assert_eq!(restored.openai_api_key, "sk-test-not-a-real-key");
        assert_eq!(restored.attached_workspace_path, original.attached_workspace_path);
    }

    #[test]
    fn deserializing_old_settings_json_without_newer_key_fields_still_works() {
        // Settings files saved by an older build won't have anthropicApiKey /
        // deepseekApiKey / xaiApiKey / togetherApiKey at all. Those fields are
        // #[serde(default)], so loading must not fail — this guards against a
        // real backward-compatibility regression.
        let legacy_json = r#"{
            "activeModel": "qwen2.5:7b",
            "attachedWorkspacePath": "/Users/someone",
            "geminiApiKey": "",
            "groqApiKey": "",
            "openrouterApiKey": "",
            "cerebrasApiKey": "",
            "sambanovaApiKey": "",
            "mistralApiKey": "",
            "openaiApiKey": ""
        }"#;

        let parsed: AppSettings = serde_json::from_str(legacy_json)
            .expect("settings.json missing newer optional keys must still parse");
        assert_eq!(parsed.anthropic_api_key, "");
        assert_eq!(parsed.deepseek_api_key, "");
        assert_eq!(parsed.xai_api_key, "");
        assert_eq!(parsed.together_api_key, "");
    }

    #[test]
    fn config_path_points_at_the_expected_app_subdirectory_and_file() {
        let path = get_config_path();
        assert_eq!(path.file_name().unwrap(), "settings.json");
        assert!(path.to_string_lossy().contains("custom-claude-coder"));
    }
}
