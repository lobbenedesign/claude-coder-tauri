use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledModel {
    pub name: String,
    pub size: u64,
    #[serde(rename = "sizeFormatted")]
    pub size_formatted: String,
    pub modified: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaTagsResponse {
    pub models: Option<Vec<OllamaModelEntry>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaModelEntry {
    pub name: String,
    pub size: u64,
    pub modified_at: Option<String>,
}

#[tauri::command]
pub async fn get_installed_ollama_models() -> Result<Vec<InstalledModel>, String> {
    let client = reqwest::Client::new();
    let res = client
        .get("http://127.0.0.1:11434/api/tags")
        .send()
        .await
        .map_err(|e| format!("Ollama non in esecuzione su localhost:11434 ({})", e))?;

    if !res.status().is_success() {
        return Err("Errore risposta Ollama".to_string());
    }

    let data: OllamaTagsResponse = res
        .json()
        .await
        .map_err(|e| format!("Errore decodifica modelli: {}", e))?;

    let mut list = Vec::new();
    if let Some(models) = data.models {
        for m in models {
            let size_gb = (m.size as f64) / (1024.0 * 1024.0 * 1024.0);
            list.push(InstalledModel {
                name: m.name,
                size: m.size,
                size_formatted: format!("{:.1} GB", size_gb),
                modified: m.modified_at.unwrap_or_else(|| "Recent".to_string()),
                status: "Pronto (Locale)".to_string(),
            });
        }
    }

    Ok(list)
}
