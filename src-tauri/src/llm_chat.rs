use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub api_base: String,
    pub api_key: String,
    pub model: String,
    pub anthropic_mode: bool,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
}

fn error_snippet(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
}

/// 发送聊天请求到 LLM API（非流式）
pub async fn chat_complete(req: ChatRequest) -> Result<ChatResponse, String> {
    if req.anthropic_mode {
        chat_anthropic(req).await
    } else {
        chat_openai(req).await
    }
}

/// Anthropic 格式 API（Claude Code / MiniMax）
async fn chat_anthropic(req: ChatRequest) -> Result<ChatResponse, String> {
    let url = format!("{}/v1/messages", req.api_base.trim_end_matches('/'));
    let max_tokens = req.max_tokens.unwrap_or(4096);

    let body = serde_json::json!({
        "model": req.model,
        "max_tokens": max_tokens,
        "messages": req.messages.iter().map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content
            })
        }).collect::<Vec<_>>(),
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("x-api-key", &req.api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("读取响应失败: {}", e))?;

    if !status.is_success() {
        return Err(format!("API 错误 ({}): {}", status.as_u16(), error_snippet(&text, 200)));
    }

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("解析响应失败: {}", e))?;

    // Anthropic 响应格式: content[0].text
    let content = json["content"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|c| c["text"].as_str())
        .unwrap_or_default()
        .to_string();

    let model = json["model"].as_str().unwrap_or(&req.model).to_string();

    Ok(ChatResponse { content, model })
}

/// OpenAI 兼容格式 API
async fn chat_openai(req: ChatRequest) -> Result<ChatResponse, String> {
    let url = format!("{}/v1/chat/completions", req.api_base.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": req.model,
        "messages": req.messages.iter().map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content
            })
        }).collect::<Vec<_>>(),
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", req.api_key))
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("读取响应失败: {}", e))?;

    if !status.is_success() {
        return Err(format!("API 错误 ({}): {}", status.as_u16(), error_snippet(&text, 200)));
    }

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("解析响应失败: {}", e))?;

    // OpenAI 响应格式: choices[0].message.content
    let content = json["choices"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|c| c["message"]["content"].as_str())
        .unwrap_or_default()
        .to_string();

    let model = json["model"].as_str().unwrap_or(&req.model).to_string();

    Ok(ChatResponse { content, model })
}
