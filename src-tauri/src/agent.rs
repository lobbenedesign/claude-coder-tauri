use crate::AgentState;
use futures_util::StreamExt;
use serde_json::json;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

/// Which LLM provider a given model identifier routes to. Extracted from the
/// if/else chain in `run_agent_stream` so the provider-selection logic (which
/// is real business logic, not UI) can be unit tested without any network
/// access or live API keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    OpenAi,
    Cerebras,
    SambaNova,
    Mistral,
    Groq,
    OpenRouter,
    Gemini,
    Ollama,
}

/// Pure routing function: given the raw model string selected in the UI,
/// decides which provider's HTTP endpoint should handle the request. Mirrors
/// exactly the prefix checks used in `run_agent_stream` (case-insensitive,
/// falls back to local Ollama when nothing else matches).
pub fn route_model_to_provider(model: &str) -> Provider {
    let model_lower = model.to_lowercase();
    if model_lower.starts_with("openai/")
        || model_lower.starts_with("gpt-")
        || model_lower.starts_with("o1")
        || model_lower.starts_with("o3")
    {
        Provider::OpenAi
    } else if model_lower.starts_with("cerebras/") {
        Provider::Cerebras
    } else if model_lower.starts_with("sambanova/") {
        Provider::SambaNova
    } else if model_lower.starts_with("mistral/") {
        Provider::Mistral
    } else if model_lower.starts_with("groq/") {
        Provider::Groq
    } else if model_lower.starts_with("openrouter/") {
        Provider::OpenRouter
    } else if model_lower.starts_with("gemini") {
        Provider::Gemini
    } else {
        Provider::Ollama
    }
}

/// Parses one line of an OpenAI-compatible SSE stream (used by OpenAI,
/// Cerebras, SambaNova, Mistral, Groq, OpenRouter) and extracts the delta
/// text content, if any. Returns `None` for non-`data:` lines, the
/// `[DONE]` sentinel, or malformed/empty-delta JSON — mirrors exactly what
/// `stream_openai_compatible` does chunk-by-chunk, but as a pure function
/// that can be unit tested with fixture payloads instead of a live HTTP
/// connection.
pub fn extract_openai_delta_content(line: &str) -> Option<String> {
    let data_str = line.strip_prefix("data: ")?;
    if data_str.trim() == "[DONE]" {
        return None;
    }
    let val: serde_json::Value = serde_json::from_str(data_str).ok()?;
    val["choices"][0]["delta"]["content"]
        .as_str()
        .map(|s| s.to_string())
}

/// Parses one line of a Gemini `streamGenerateContent` SSE stream and
/// extracts the candidate text, if any. Mirrors the parsing inline in
/// `run_agent_stream`'s Gemini branch.
pub fn extract_gemini_chunk_text(line: &str) -> Option<String> {
    let data_str = line.strip_prefix("data: ")?;
    let val: serde_json::Value = serde_json::from_str(data_str).ok()?;
    val["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .map(|s| s.to_string())
}

/// Parses one line of Ollama's newline-delimited JSON chat stream and
/// extracts the message content, if any. Mirrors the parsing inline in
/// `run_agent_stream`'s Ollama branch.
pub fn extract_ollama_message_content(line: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(line).ok()?;
    val["message"]["content"].as_str().map(|s| s.to_string())
}

/// Signals the currently in-flight `run_agent_stream` call (if any) to stop.
/// This is a real abort: the streaming loops in this file poll the flag
/// between chunks and break out of the HTTP stream as soon as it flips,
/// instead of merely hiding the "stop" button in the UI.
#[tauri::command]
pub fn stop_agent_stream(state: State<'_, AgentState>) -> Result<bool, String> {
    state.cancel_flag.store(true, Ordering::Relaxed);
    Ok(true)
}

#[tauri::command]
pub async fn run_agent_stream(
    app: AppHandle,
    state: State<'_, AgentState>,
    prompt: String,
    model: String,
    workspace: String,
    repo_map: String,
    gemini_key: String,
    groq_key: String,
    openrouter_key: String,
    cerebras_key: String,
    sambanova_key: String,
    mistral_key: String,
    openai_key: String,
) -> Result<bool, String> {
    let client = reqwest::Client::new();

    // Reset the cancellation flag for this fresh run, and hand each provider
    // path a clone of the shared Arc so it can be polled mid-stream.
    state.cancel_flag.store(false, Ordering::Relaxed);
    let cancel_flag = state.cancel_flag.clone();

    let repo_map_block = if repo_map.trim().is_empty() {
        String::new()
    } else {
        format!("\n\nContesto struttura del progetto (repo map reale, estratta dai file su disco):\n{}\n", repo_map)
    };

    let system_prompt = format!(
        "Sei CUSTOM CLAUDE CODER (v2.1 Rust Desktop), l'assistente AI avanzato di programmazione.
Workspace attivo: '{}'.
Fornisci codice pulito, spiegazioni passo-passo e suggerimenti di qualità per questo progetto.
Quando proponi una modifica a un file esistente, esprimila SEMPRE come un blocco di codice ```diff
contenente un unified diff valido (righe --- a/<percorso relativo>, +++ b/<percorso relativo>, hunk @@),
cosi' l'utente puo' rivederla e applicarla con un click invece di copiare/incollare codice a mano.{}",
        workspace, repo_map_block
    );

    // 0. OPENAI / CHATGPT
    if route_model_to_provider(&model) == Provider::OpenAi {
        if openai_key.is_empty() {
            return Err("Chiave OpenAI API (ChatGPT) non configurata! Inseriscila nella scheda API Keys.".to_string());
        }
        let real_model = model.strip_prefix("openai/").unwrap_or(&model);
        stream_openai_compatible(
            &app,
            &client,
            cancel_flag.clone(),
            "https://api.openai.com/v1/chat/completions",
            &openai_key,
            real_model,
            &system_prompt,
            &prompt,
        )
        .await?;
        return Ok(true);
    }

    // 1. CEREBRAS CLOUD
    if route_model_to_provider(&model) == Provider::Cerebras {
        if cerebras_key.is_empty() {
            return Err("Chiave Cerebras API non configurata! Inseriscila nella scheda API Keys.".to_string());
        }
        let real_model = model.strip_prefix("cerebras/").unwrap_or(&model);
        stream_openai_compatible(
            &app,
            &client,
            cancel_flag.clone(),
            "https://api.cerebras.ai/v1/chat/completions",
            &cerebras_key,
            real_model,
            &system_prompt,
            &prompt,
        )
        .await?;
        return Ok(true);
    }

    // 2. SAMBANOVA CLOUD
    if route_model_to_provider(&model) == Provider::SambaNova {
        if sambanova_key.is_empty() {
            return Err("Chiave SambaNova API non configurata! Inseriscila nella scheda API Keys.".to_string());
        }
        let real_model = model.strip_prefix("sambanova/").unwrap_or(&model);
        stream_openai_compatible(
            &app,
            &client,
            cancel_flag.clone(),
            "https://api.sambanova.ai/v1/chat/completions",
            &sambanova_key,
            real_model,
            &system_prompt,
            &prompt,
        )
        .await?;
        return Ok(true);
    }

    // 3. MISTRAL AI (CODESTRAL)
    if route_model_to_provider(&model) == Provider::Mistral {
        if mistral_key.is_empty() {
            return Err("Chiave Mistral API non configurata! Inseriscila nella scheda API Keys.".to_string());
        }
        let real_model = model.strip_prefix("mistral/").unwrap_or(&model);
        stream_openai_compatible(
            &app,
            &client,
            cancel_flag.clone(),
            "https://api.mistral.ai/v1/chat/completions",
            &mistral_key,
            real_model,
            &system_prompt,
            &prompt,
        )
        .await?;
        return Ok(true);
    }

    // 4. GROQ CLOUD
    if route_model_to_provider(&model) == Provider::Groq {
        if groq_key.is_empty() {
            return Err("Chiave Groq API non configurata! Inseriscila nella scheda API Keys.".to_string());
        }
        let real_model = model.strip_prefix("groq/").unwrap_or(&model);
        stream_openai_compatible(
            &app,
            &client,
            cancel_flag.clone(),
            "https://api.groq.com/openai/v1/chat/completions",
            &groq_key,
            real_model,
            &system_prompt,
            &prompt,
        )
        .await?;
        return Ok(true);
    }

    // 5. OPENROUTER
    if route_model_to_provider(&model) == Provider::OpenRouter {
        if openrouter_key.is_empty() {
            return Err("Chiave OpenRouter API non configurata! Inseriscila nella scheda API Keys.".to_string());
        }
        let real_model = model.strip_prefix("openrouter/").unwrap_or(&model);
        stream_openai_compatible(
            &app,
            &client,
            cancel_flag.clone(),
            "https://openrouter.ai/api/v1/chat/completions",
            &openrouter_key,
            real_model,
            &system_prompt,
            &prompt,
        )
        .await?;
        return Ok(true);
    }

    // 6. GOOGLE GEMINI
    if route_model_to_provider(&model) == Provider::Gemini {
        if gemini_key.is_empty() {
            return Err("Chiave Google Gemini API non configurata! Inseriscila nella scheda API Keys.".to_string());
        }
        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?key={}&alt=sse",
            model, gemini_key
        );

        let body = json!({
            "contents": [{ "role": "user", "parts": [{ "text": prompt }] }],
            "systemInstruction": { "parts": [{ "text": system_prompt }] },
            "generationConfig": { "temperature": 0.7, "maxOutputTokens": 8192 }
        });

        let mut stream = client
            .post(&endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .bytes_stream();

        while let Some(chunk) = stream.next().await {
            if cancel_flag.load(Ordering::Relaxed) {
                let _ = app.emit("agent-stopped", ());
                return Ok(true);
            }
            if let Ok(bytes) = chunk {
                let text = String::from_utf8_lossy(&bytes);
                for line in text.lines() {
                    if let Some(part_text) = extract_gemini_chunk_text(line) {
                        let _ = app.emit("agent-chunk", part_text);
                    }
                }
            }
        }
        let _ = app.emit("agent-done", ());
        return Ok(true);
    }

    // 7. LOCAL OLLAMA
    let ollama_payload = json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": prompt }
        ],
        "stream": true
    });

    let res = client
        .post("http://127.0.0.1:11434/api/chat")
        .json(&ollama_payload)
        .send()
        .await
        .map_err(|e| format!("Errore connessione Ollama (localhost:11434): {}", e))?;

    let mut stream = res.bytes_stream();
    while let Some(chunk) = stream.next().await {
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = app.emit("agent-stopped", ());
            return Ok(true);
        }
        if let Ok(bytes) = chunk {
            let text = String::from_utf8_lossy(&bytes);
            for line in text.lines() {
                if let Some(msg_content) = extract_ollama_message_content(line) {
                    let _ = app.emit("agent-chunk", msg_content);
                }
            }
        }
    }

    let _ = app.emit("agent-done", ());
    Ok(true)
}

async fn stream_openai_compatible(
    app: &AppHandle,
    client: &reqwest::Client,
    cancel_flag: Arc<std::sync::atomic::AtomicBool>,
    endpoint: &str,
    api_key: &str,
    model_id: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<(), String> {
    let payload = json!({
        "model": model_id,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ],
        "temperature": 0.7,
        "stream": true
    });

    let res = client
        .post(endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("HTTP-Referer", "https://customclaudecoder.app")
        .header("X-Title", "Custom Claude Coder")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Errore provider: {}", err_text));
    }

    let mut stream = res.bytes_stream();
    while let Some(chunk) = stream.next().await {
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = app.emit("agent-stopped", ());
            return Ok(());
        }
        if let Ok(bytes) = chunk {
            let text = String::from_utf8_lossy(&bytes);
            for line in text.lines() {
                if let Some(content) = extract_openai_delta_content(line) {
                    let _ = app.emit("agent-chunk", content);
                }
            }
        }
    }

    let _ = app.emit("agent-done", ());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Provider routing (which of the 8 LLM providers a model id maps to) ---

    #[test]
    fn routes_openai_style_prefixes_to_openai() {
        assert_eq!(route_model_to_provider("openai/gpt-4o"), Provider::OpenAi);
        assert_eq!(route_model_to_provider("gpt-4o-mini"), Provider::OpenAi);
        assert_eq!(route_model_to_provider("o1-preview"), Provider::OpenAi);
        assert_eq!(route_model_to_provider("o3-mini"), Provider::OpenAi);
        // Case-insensitive, matches the .to_lowercase() done in run_agent_stream.
        assert_eq!(route_model_to_provider("OpenAI/GPT-4O"), Provider::OpenAi);
    }

    #[test]
    fn routes_cloud_provider_prefixes() {
        assert_eq!(route_model_to_provider("cerebras/llama-3.3-70b"), Provider::Cerebras);
        assert_eq!(route_model_to_provider("sambanova/Meta-Llama-3.1-405B"), Provider::SambaNova);
        assert_eq!(route_model_to_provider("mistral/codestral-latest"), Provider::Mistral);
        assert_eq!(route_model_to_provider("groq/llama-3.1-8b-instant"), Provider::Groq);
        assert_eq!(route_model_to_provider("openrouter/anthropic/claude-3.5-sonnet"), Provider::OpenRouter);
    }

    #[test]
    fn routes_gemini_prefix_case_insensitively() {
        assert_eq!(route_model_to_provider("gemini-1.5-pro"), Provider::Gemini);
        assert_eq!(route_model_to_provider("Gemini-2.0-Flash"), Provider::Gemini);
    }

    #[test]
    fn unrecognized_model_falls_back_to_local_ollama() {
        assert_eq!(route_model_to_provider("qwen2.5:7b"), Provider::Ollama);
        assert_eq!(route_model_to_provider("llama3.1:8b"), Provider::Ollama);
        assert_eq!(route_model_to_provider(""), Provider::Ollama);
    }

    // --- OpenAI-compatible SSE delta parsing (OpenAI, Cerebras, SambaNova, Mistral, Groq, OpenRouter) ---

    #[test]
    fn extracts_delta_content_from_openai_compatible_sse_line() {
        let line = r#"data: {"choices":[{"delta":{"content":"Ciao"}}]}"#;
        assert_eq!(extract_openai_delta_content(line), Some("Ciao".to_string()));
    }

    #[test]
    fn openai_compatible_done_sentinel_yields_no_content() {
        assert_eq!(extract_openai_delta_content("data: [DONE]"), None);
    }

    #[test]
    fn openai_compatible_non_data_line_yields_no_content() {
        assert_eq!(extract_openai_delta_content(": keep-alive"), None);
        assert_eq!(extract_openai_delta_content(""), None);
    }

    #[test]
    fn openai_compatible_malformed_json_yields_no_content_not_a_panic() {
        assert_eq!(extract_openai_delta_content("data: {not valid json"), None);
    }

    #[test]
    fn openai_compatible_role_only_chunk_yields_no_content() {
        // First SSE chunk of a stream is typically just {"delta":{"role":"assistant"}}
        // with no "content" key yet — must not be mistaken for empty text.
        let line = r#"data: {"choices":[{"delta":{"role":"assistant"}}]}"#;
        assert_eq!(extract_openai_delta_content(line), None);
    }

    // --- Gemini SSE parsing ---

    #[test]
    fn extracts_text_from_gemini_sse_line() {
        let line = r#"data: {"candidates":[{"content":{"parts":[{"text":"Buongiorno"}]}}]}"#;
        assert_eq!(extract_gemini_chunk_text(line), Some("Buongiorno".to_string()));
    }

    #[test]
    fn gemini_line_without_data_prefix_yields_no_text() {
        assert_eq!(extract_gemini_chunk_text("{}"), None);
    }

    #[test]
    fn gemini_malformed_json_yields_no_text_not_a_panic() {
        assert_eq!(extract_gemini_chunk_text("data: not json at all"), None);
    }

    // --- Ollama NDJSON parsing ---

    #[test]
    fn extracts_message_content_from_ollama_ndjson_line() {
        let line = r#"{"model":"qwen2.5:7b","message":{"role":"assistant","content":"Salve"},"done":false}"#;
        assert_eq!(extract_ollama_message_content(line), Some("Salve".to_string()));
    }

    #[test]
    fn ollama_done_line_without_message_yields_no_content() {
        let line = r#"{"model":"qwen2.5:7b","done":true,"total_duration":12345}"#;
        assert_eq!(extract_ollama_message_content(line), None);
    }

    #[test]
    fn ollama_malformed_json_yields_no_content_not_a_panic() {
        assert_eq!(extract_ollama_message_content("not json"), None);
    }
}
