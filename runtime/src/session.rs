use serde::Serialize;
use std::collections::HashMap;
use crate::context::Message;
use crate::tools::ToolRegistry;
use crate::memory::Memory;

/// 会话
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub messages: Vec<Message>,
    pub created_at: u64,
    pub updated_at: u64,
    pub turn_count: u32,
    pub max_turns: u32,
    pub memory: Memory,
}

/// 系统提示词（ReAct 风格）
pub fn system_prompt() -> String {
    let tool_desc = ToolRegistry::new().description();
    format!(
        "你是一个智能助手，可以使用工具来帮助用户。\n\
         \n\
         {}\n\
         回复必须是以下 JSON 格式之一：\n\
         \n\
         1. 使用工具（可以多次使用）：\n\
         {{\"type\": \"tool_call\", \"tool\": \"工具名\", \"params\": {{\"参数名\": \"参数值\"}}, \"thought\": \"为什么用这个工具\"}}\n\
         \n\
         2. 任务已完成：\n\
         {{\"type\": \"final_answer\", \"content\": \"最终答案内容\"}}\n\
         \n\
         3. 需要向用户提问：\n\
         {{\"type\": \"ask_user\", \"question\": \"你的问题\"}}\n\
         \n\
         规则：\n\
         - 收集到足够信息后，必须给出 final_answer\n\
         - 不要用相同参数重复调用同一个工具\n\
         - 如果一次需要多个工具，连续输出多行 JSON\n\
         - 参数名必须和工具定义完全一致（英文）\n\
         - 只返回 JSON，不要加任何解释",
        tool_desc
    )
}

impl Session {
    pub fn new(name: String) -> Self {
        let now = unix_ms();
        let mut memory = Memory::new();
        memory.set_system(Message {
            role: "system".to_string(),
            content: system_prompt(),
            tool_call_id: None,
            tool_name: None,
        });
        Self {
            id: generate_id(),
            name,
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
            turn_count: 0,
            max_turns: 50,
            memory,
        }
    }

    /// 同步 messages 从 memory（用于序列化返回给前端）
    pub fn sync(&mut self) {
        self.messages = self.memory.short_term.clone();
    }

    /// 序列化为 JSON（含 messages）
    pub fn to_json(&self) -> String {
        #[derive(Serialize)]
        struct SessionJson<'a> {
            id: &'a str,
            name: &'a str,
            messages: &'a [Message],
            created_at: u64,
            updated_at: u64,
            turn_count: u32,
            max_turns: u32,
        }
        let json = SessionJson {
            id: &self.id,
            name: &self.name,
            messages: &self.memory.short_term,
            created_at: self.created_at,
            updated_at: self.updated_at,
            turn_count: self.turn_count,
            max_turns: self.max_turns,
        };
        serde_json::to_string(&json).unwrap_or_default()
    }

    pub fn is_expired(&self) -> bool {
        self.turn_count >= self.max_turns
    }

    /// 压缩记忆
    pub fn compress(&mut self) {
        self.memory.compress();
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
