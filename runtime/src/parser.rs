use serde::{Deserialize, Serialize};

/// 解析后的 action
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentAction {
    pub action: String,
    #[serde(flatten)]
    pub args: serde_json::Map<String, serde_json::Value>,
}

/// 从 LLM 响应中提取所有 JSON action
/// LLM 回复中只有 JSON 是有效内容，其余都是幻觉
/// 支持格式：
///   {"action": "answer", "content": "..."}
///   {"action": "use_tool", "tool": "calculator", "params": {...}}
///   也兼容旧格式：{"action": "calculator", "expression": "..."}
pub fn parse_actions(raw: &str) -> Vec<AgentAction> {
    let text = extract_text(raw);
    let jsons = extract_json_objects(&text);

    let mut actions = Vec::new();
    for json_str in jsons {
        match parse_single_action(&json_str) {
            Ok(action) => actions.push(action),
            Err(_) => continue, // 解析失败的 JSON 跳过
        }
    }
    actions
}

/// 从文本中提取所有顶级 {} 包裹的 JSON 字符串
fn extract_json_objects(text: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;

    for (i, ch) in text.char_indices() {
        match ch {
            '{' => {
                if depth == 0 { start = i; }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    results.push(text[start..=i].to_string());
                }
            }
            _ => {}
        }
    }
    results
}

/// 解析单个 JSON action
fn parse_single_action(json_str: &str) -> Result<AgentAction, String> {
    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;

    let action = parsed.get("action").and_then(|a| a.as_str())
        .ok_or_else(|| "缺少 action 字段".to_string())?;

    let mut args = serde_json::Map::new();

    match action {
        "use_tool" => {
            let tool_name = parsed.get("tool").and_then(|t| t.as_str())
                .ok_or_else(|| "use_tool 缺少 tool 字段".to_string())?;
            if let Some(params) = parsed.get("params").and_then(|p| p.as_object()) {
                for (k, v) in params { args.insert(k.clone(), v.clone()); }
            }
            return Ok(AgentAction { action: tool_name.to_string(), args });
        }
        "answer" => {
            let content = parsed.get("content").and_then(|c| c.as_str()).unwrap_or("");
            args.insert("content".to_string(), serde_json::Value::String(content.to_string()));
        }
        _ => {
            // 旧格式：{"action": "calculator", "expression": "..."}
            if let Some(obj) = parsed.as_object() {
                for (k, v) in obj {
                    if k != "action" { args.insert(k.clone(), v.clone()); }
                }
            }
        }
    }

    Ok(AgentAction { action: action.to_string(), args })
}

/// 提取 think/reasoning 标签内的思考内容（用于日志展示）
pub fn extract_think(raw: &str) -> Option<String> {
    let text = if let Ok(val) = serde_json::from_str::<serde_json::Value>(raw) {
        if let Some(choices) = val.get("choices").and_then(|c| c.as_array()) {
            if let Some(choice) = choices.first() {
                if let Some(msg) = choice.get("message") {
                    msg.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string()
                } else { return None; }
            } else { return None; }
        } else { return None; }
    } else { return None; };

    for tag in &["think", "reasoning"] {
        let open = format!("<{}>", tag);
        let close = format!("</{}>", tag);
        if let Some(start) = text.find(&open) {
            let from = start + open.len();
            if let Some(end) = text[from..].find(&close) {
                let content = text[from..from + end].trim().to_string();
                if !content.is_empty() { return Some(content); }
            }
        }
    }
    None
}

/// 提取纯文本（去掉 think 标签和 markdown）
pub fn extract_text_content(raw: &str) -> String {
    let text = if let Ok(val) = serde_json::from_str::<serde_json::Value>(raw) {
        if let Some(choices) = val.get("choices").and_then(|c| c.as_array()) {
            if let Some(choice) = choices.first() {
                if let Some(msg) = choice.get("message") {
                    msg.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string()
                } else { raw.to_string() }
            } else { raw.to_string() }
        } else { raw.to_string() }
    } else { raw.to_string() };
    strip_think_tags(&strip_markdown(&text))
}

fn extract_text(raw: &str) -> String {
    let text = if let Ok(val) = serde_json::from_str::<serde_json::Value>(raw) {
        if let Some(choices) = val.get("choices").and_then(|c| c.as_array()) {
            if let Some(choice) = choices.first() {
                if let Some(msg) = choice.get("message") {
                    msg.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string()
                } else { raw.to_string() }
            } else { raw.to_string() }
        } else { raw.to_string() }
    } else { raw.to_string() };
    strip_think_tags(&strip_markdown(&text))
}

fn strip_think_tags(s: &str) -> String {
    let mut result = s.to_string();
    for tag in &["think", "reasoning"] {
        let open = format!("<{}>", tag);
        let close = format!("</{}>", tag);
        while let Some(start) = result.find(&open) {
            match result[start..].find(&close) {
                Some(end) => { result.drain(start..start + end + close.len()); }
                None => { result.drain(start..start + open.len()); }
            }
        }
    }
    result
}

fn strip_markdown(s: &str) -> String {
    let s = s.trim();
    let s = s.strip_prefix("```json").or_else(|| s.strip_prefix("```")).unwrap_or(s)
        .strip_suffix("```").unwrap_or(s).trim();
    s.to_string()
}
