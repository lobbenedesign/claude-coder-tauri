use futures_util::StreamExt;
use serde_json::json;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn run_agent_stream(
    app: AppHandle,
    prompt: String,
    model: String,
    workspace: String,
    gemini_key: String,
    groq_key: String,
    openrouter_key: String,
    cerebras_key: String,
    sambanova_key: String,
    mistral_key: String,
    openai_key: String,
) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let model_lower = model.to_lowercase();

    let system_prompt = format!(
        "Sei CUSTOM CLAUDE CODER (v2.1 Rust Desktop), l'assistente AI avanzato di programmazione.
Workspace attivo: '{}'.
Fornisci codice pulito, spiegazioni passo-passo e suggerimenti di qualità per questo progetto.",
        workspace
    );

    // 0. OPENAI / CHATGPT
    if model_lower.starts_with("openai/") || model_lower.starts_with("gpt-") || model_lower.starts_with("o1") || model_lower.starts_with("o3") {
        if openai_key.is_empty() {
            return Err("Chiave OpenAI API (ChatGPT) non configurata! Inseriscila nella scheda API Keys.".to_string());
        }
        let real_model = model.strip_prefix("openai/").unwrap_or(&model);
        stream_openai_compatible(
            &app,
            &client,
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
    if model_lower.starts_with("cerebras/") {
        if cerebras_key.is_empty() {
            return Err("Chiave Cerebras API non configurata! Inseriscila nella scheda API Keys.".to_string());
        }
        let real_model = model.strip_prefix("cerebras/").unwrap_or(&model);
        stream_openai_compatible(
            &app,
            &client,
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
    if model_lower.starts_with("sambanova/") {
        if sambanova_key.is_empty() {
            return Err("Chiave SambaNova API non configurata! Inseriscila nella scheda API Keys.".to_string());
        }
        let real_model = model.strip_prefix("sambanova/").unwrap_or(&model);
        stream_openai_compatible(
            &app,
            &client,
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
    if model_lower.starts_with("mistral/") {
        if mistral_key.is_empty() {
            return Err("Chiave Mistral API non configurata! Inseriscila nella scheda API Keys.".to_string());
        }
        let real_model = model.strip_prefix("mistral/").unwrap_or(&model);
        stream_openai_compatible(
            &app,
            &client,
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
    if model_lower.starts_with("groq/") {
        if groq_key.is_empty() {
            return Err("Chiave Groq API non configurata! Inseriscila nella scheda API Keys.".to_string());
        }
        let real_model = model.strip_prefix("groq/").unwrap_or(&model);
        stream_openai_compatible(
            &app,
            &client,
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
    if model_lower.starts_with("openrouter/") {
        if openrouter_key.is_empty() {
            return Err("Chiave OpenRouter API non configurata! Inseriscila nella scheda API Keys.".to_string());
        }
        let real_model = model.strip_prefix("openrouter/").unwrap_or(&model);
        stream_openai_compatible(
            &app,
            &client,
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
    if model_lower.starts_with("gemini") {
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
            if let Ok(bytes) = chunk {
                let text = String::from_utf8_lossy(&bytes);
                for line in text.lines() {
                    if let Some(data_str) = line.strip_prefix("data: ") {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(data_str) {
                            if let Some(part_text) = val["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                                let _ = app.emit("agent-chunk", part_text);
                            }
                        }
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
        if let Ok(bytes) = chunk {
            let text = String::from_utf8_lossy(&bytes);
            for line in text.lines() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(msg_content) = val["message"]["content"].as_str() {
                        let _ = app.emit("agent-chunk", msg_content);
                    }
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
        if let Ok(bytes) = chunk {
            let text = String::from_utf8_lossy(&bytes);
            for line in text.lines() {
                if let Some(data_str) = line.strip_prefix("data: ") {
                    if data_str.trim() == "[DONE]" {
                        continue;
                    }
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(data_str) {
                        if let Some(content) = val["choices"][0]["delta"]["content"].as_str() {
                            let _ = app.emit("agent-chunk", content);
                        }
                    }
                }
            }
        }
    }

    let _ = app.emit("agent-done", ());
    Ok(())
}
