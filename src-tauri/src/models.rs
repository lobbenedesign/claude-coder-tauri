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

/// Formats a raw byte count (as reported by Ollama's `/api/tags`) into the
/// human-readable "X.X GB" string shown in the model list. Extracted from
/// `get_installed_ollama_models` so the formatting logic can be unit tested
/// with fixture byte counts, without needing a live local Ollama server.
pub fn format_size_gb(bytes: u64) -> String {
    let size_gb = (bytes as f64) / (1024.0 * 1024.0 * 1024.0);
    format!("{:.1} GB", size_gb)
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
            list.push(InstalledModel {
                name: m.name,
                size: m.size,
                size_formatted: format_size_gb(m.size),
                modified: m.modified_at.unwrap_or_else(|| "Recent".to_string()),
                status: "Pronto (Locale)".to_string(),
            });
        }
    }

    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_exact_gigabyte_boundary() {
        assert_eq!(format_size_gb(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn formats_zero_bytes() {
        assert_eq!(format_size_gb(0), "0.0 GB");
    }

    #[test]
    fn formats_a_realistic_model_size() {
        // qwen2.5:7b is roughly 4.7 GB on disk.
        let bytes: u64 = 4_700_000_000;
        let formatted = format_size_gb(bytes);
        assert!(formatted.starts_with("4."), "expected ~4.x GB, got {}", formatted);
        assert!(formatted.ends_with(" GB"));
    }

    #[test]
    fn formats_sub_gigabyte_size_without_rounding_up_to_a_whole_gb() {
        let bytes: u64 = 100 * 1024 * 1024; // 100 MB
        assert_eq!(format_size_gb(bytes), "0.1 GB");
    }
}
