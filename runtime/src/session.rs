use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::context::Message;
use crate::tools::ToolRegistry;

/// 会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub messages: Vec<Message>,
    pub created_at: u64,
    pub updated_at: u64,
    pub turn_count: u32,
    pub max_turns: u32,
}

/// 系统提示词
pub fn system_prompt() -> String {
    let tool_desc = ToolRegistry::new().description();
    format!(
        "你是一个智能助手，可以使用工具来帮助用户。\n\
         \n\
         {}\n\
         回复格式要求：\n\
         \n\
         1. 直接回答：\n\
         {{\"action\": \"answer\", \"content\": \"你的回答\"}}\n\
         \n\
         2. 需要调用工具：\n\
         {{\"action\": \"use_tool\", \"tool\": \"工具名\", \"params\": {{\"参数名\": \"参数值\"}}}}\n\
         \n\
         3. 需要同时做多件事时，连续输出多个 JSON，每行一个：\n\
         {{\"action\": \"use_tool\", \"tool\": \"工具名\", \"params\": {{\"参数名\": \"参数值\"}}}}\n\
         {{\"action\": \"use_tool\", \"tool\": \"工具名\", \"params\": {{\"参数名\": \"参数值\"}}}}\n\
         \n\
         参数名必须和工具定义完全一致（英文）。\n\
         只返回 JSON，不要加任何解释。\n\
         不要使用 think/reasoning 标签。",
        tool_desc
    )
}

impl Session {
    pub fn new(name: String) -> Self {
        let now = unix_ms();
        Self {
            id: generate_id(),
            name,
            messages: vec![Message {
                role: "system".to_string(),
                content: system_prompt(),
                tool_call_id: None,
                tool_name: None,
            }],
            created_at: now,
            updated_at: now,
            turn_count: 0,
            max_turns: 50,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.turn_count >= self.max_turns
    }

    pub fn compress(&mut self) {
        if self.messages.len() <= 41 {
            return;
        }
        let mut system = Vec::new();
        let mut rest = Vec::new();
        for m in self.messages.drain(..) {
            if m.role == "system" {
                system.push(m);
            } else {
                rest.push(m);
            }
        }
        let keep = rest.split_off(rest.len().saturating_sub(40));
        let summary = Message {
            role: "assistant".to_string(),
            content: format!("[上下文已压缩，省略了之前 {} 条消息]", rest.len()),
            tool_call_id: None,
            tool_name: None,
        };
        self.messages = system;
        self.messages.push(summary);
        self.messages.extend(keep);
        self.updated_at = unix_ms();
    }
}

/// Session 管理器
pub struct SessionManager {
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self { sessions: HashMap::new() }
    }

    pub fn create(&mut self, name: String) -> &mut Session {
        let session = Session::new(name);
        let id = session.id.clone();
        self.sessions.entry(id).or_insert(session)
    }

    pub fn delete(&mut self, id: &str) {
        self.sessions.remove(id);
    }

    pub fn get(&self, id: &str) -> Option<&Session> {
        self.sessions.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(id)
    }

    pub fn list(&self) -> Vec<&Session> {
        let mut list: Vec<&Session> = self.sessions.values().collect();
        list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        list
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

use std::sync::atomic::{AtomicU64, Ordering};

fn generate_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let ms = unix_ms();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("sess_{:x}_{:04x}", ms, seq % 10000)
}

fn unix_ms() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
    }
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64
    }
}
