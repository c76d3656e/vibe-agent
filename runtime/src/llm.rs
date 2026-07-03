use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use crate::context::Message;

/// 构建 OpenAI 兼容的请求体（纯文本，不加 tools 参数）
pub fn build_request(messages: &[Message], model: &str) -> String {
    let body = serde_json::json!({
        "model": model,
        "messages": messages.iter().map(|m| {
            let mut msg = serde_json::json!({
                "role": m.role,
                "content": m.content,
            });
            if m.role == "tool" {
                msg["tool_call_id"] = serde_json::json!(m.tool_call_id);
            }
            msg
        }).collect::<Vec<_>>(),
    });

    serde_json::to_string(&body).unwrap_or_default()
}

/// 通过 web-sys fetch 调用 LLM API（通过同源代理解决 CORS）
pub async fn call_llm(request_body: &str, api_url: &str, api_key: &str) -> Result<String, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("无 window 对象"))?;

    // 自动补全 URL 路径
    let full_url = normalize_url(api_url);

    // 构造代理请求体：{ url, key, body }
    let parsed_body: serde_json::Value = serde_json::from_str(request_body).unwrap_or_default();
    let proxy_body = serde_json::json!({
        "url": full_url,
        "key": api_key,
        "body": parsed_body,
    });
    let proxy_body_str = serde_json::to_string(&proxy_body).unwrap_or_default();

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");

    let headers = web_sys::Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    opts.set_headers(&headers);

    opts.set_body(&JsValue::from_str(&proxy_body_str));

    // 同源代理路径
    let request = web_sys::Request::new_with_str_and_init("/api/llm", &opts)?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp_value.dyn_into()?;

    if !resp.ok() {
        let status = resp.status();
        return Err(JsValue::from_str(&format!("HTTP {}: {}", status, resp.status_text())));
    }

    let text_promise = resp.text()?;
    let text_value = JsFuture::from(text_promise).await?;
    text_value.as_string().ok_or_else(|| JsValue::from_str("响应为空"))
}

/// 自动补全 API URL 路径
/// 支持格式：ai.gitee.com → https://ai.gitee.com/v1/chat/completions
///           ai.gitee.com/v1 → https://ai.gitee.com/v1/chat/completions
///           https://ai.gitee.com/v1 → https://ai.gitee.com/v1/chat/completions
///           https://api.openai.com/v1/chat/completions → 不变
fn normalize_url(url: &str) -> String {
    let mut s = url.trim().to_string();

    // 补 https://
    if !s.starts_with("http://") && !s.starts_with("https://") {
        s = format!("https://{}", s);
    }

    // 去掉末尾 /
    s = s.trim_end_matches('/').to_string();

    // 如果已经包含完整路径，直接返回
    if s.ends_with("/chat/completions") {
        return s;
    }

    // 如果以 /v1 结尾，补 /chat/completions
    if s.ends_with("/v1") {
        return format!("{}/chat/completions", s);
    }

    // 其他情况：尝试补 /v1/chat/completions
    format!("{}/v1/chat/completions", s)
}
