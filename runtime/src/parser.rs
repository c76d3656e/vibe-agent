use serde::{Deserialize, Serialize};

/// 解析后的 action
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentAction {
    pub action: String,        // "final_answer" | "tool_call" | "ask_user"
    pub args: serde_json::Map<String, serde_json::Value>,
}

/// 从 LLM 响应中提取所有 JSON action
/// 支持格式：
///   {"type": "final_answer", "content": "..."}
///   {"type": "tool_call", "tool": "calculator", "params": {...}, "thought": "..."}
///   {"type": "ask_user", "question": "..."}
///   兼容旧格式：{"action": "answer", "content": "..."}
pub fn parse_actions(raw: &str) -> Vec<AgentAction> {
    let text = extract_text(raw);
    let json_strs = extract_json_objects(&text);

    // 如果没找到，尝试 regex 风格兜底（找第一个完整 {}）
    let json_strs = if json_strs.is_empty() {
        fallback_extract_json(&text)
    } else {
        json_strs
    };

    let mut actions = Vec::new();
    for s in json_strs {
        match parse_single(&s) {
            Ok(a) => actions.push(a),
            Err(_) => continue,
        }
    }
    actions
}

/// 提取所有顶级 {} 包裹的 JSON 字符串
fn extract_json_objects(text: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;

    for (i, ch) in text.char_indices() {
        match ch {
            '{' => { if depth == 0 { start = i; } depth += 1; }
            '}' => { depth -= 1; if depth == 0 { results.push(text[start..=i].to_string()); } }
            _ => {}
        }
    }
    results
}

/// fallback：在文本中找第一个 { ... } 块（类似 regex search）
fn fallback_extract_json(text: &str) -> Vec<String> {
    if let Some(start) = text.find('{') {
        let mut depth = 0i32;
        for (i, ch) in text[start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => { depth -= 1; if depth == 0 { return vec![text[start..start + i + 1].to_string()]; } }
                _ => {}
            }
        }
    }
    Vec::new()
}

/// 解析单个 JSON 为 AgentAction
fn parse_single(json_str: &str) -> Result<AgentAction, String> {
    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|_| "JSON 解析失败".to_string())?;

    // 判断使用 type 还是 action 字段
    let type_field = parsed.get("type").and_then(|t| t.as_str())
        .or_else(|| parsed.get("action").and_then(|a| a.as_str()))
        .ok_or_else(|| "缺少 type/action 字段".to_string())?;

    let mut args = serde_json::Map::new();

    match type_field {
        "tool_call" | "use_tool" => {
            // {"type": "tool_call", "tool": "calc", "params": {...}, "thought": "..."}
            // 兼容旧: {"action": "use_tool", "tool": "calc", "params": {...}}
            let tool_name = parsed.get("tool").and_then(|t| t.as_str())
                .ok_or_else(|| "tool_call 缺少 tool 字段".to_string())?;
            if let Some(params) = parsed.get("params").and_then(|p| p.as_object()) {
                for (k, v) in params { args.insert(k.clone(), v.clone()); }
            }
            // thought 字段记入 args 以便 trace 展示
            if let Some(thought) = parsed.get("thought").and_then(|t| t.as_str()) {
                args.insert("__thought__".to_string(), serde_json::Value::String(thought.to_string()));
            }
            return Ok(AgentAction { action: tool_name.to_string(), args });
        }
        "final_answer" | "answer" => {
            // {"type": "final_answer", "content": "..."}
            let content = parsed.get("content").and_then(|c| c.as_str()).unwrap_or("");
            args.insert("content".to_string(), serde_json::Value::String(content.to_string()));
        }
        "ask_user" => {
            // {"type": "ask_user", "question": "..."}
            let question = parsed.get("question").and_then(|q| q.as_str()).unwrap_or("");
            args.insert("question".to_string(), serde_json::Value::String(question.to_string()));
        }
        _ => {
            // 旧格式：{"action": "calculator", "expression": "..."}
            if let Some(obj) = parsed.as_object() {
                for (k, v) in obj {
                    if k != "action" && k != "type" { args.insert(k.clone(), v.clone()); }
                }
            }
        }
    }

    Ok(AgentAction { action: type_field.to_string(), args })
}

/// 提取 think/reasoning 标签内的思考内容（用于日志展示）
pub fn extract_think(raw: &str) -> Option<String> {
    let text = extract_raw_content(raw);
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
    let text = extract_raw_content(raw);
    strip_think_tags(&strip_markdown(&text))
}

fn extract_raw_content(raw: &str) -> String {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(raw) {
        if let Some(choices) = val.get("choices").and_then(|c| c.as_array()) {
            if let Some(choice) = choices.first() {
                if let Some(msg) = choice.get("message") {
                    return msg.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string();
                }
            }
        }
    }
    raw.to_string()
}

fn extract_text(raw: &str) -> String {
    strip_think_tags(&strip_markdown(&extract_raw_content(raw)))
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
