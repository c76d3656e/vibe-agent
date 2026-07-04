use serde::{Deserialize, Serialize};
use crate::context::Message;

const MAX_SHORT_TERM: usize = 41; // system + 40 条消息

/// 记忆系统（当前仅短期记忆）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// 短期记忆：当前轮次的对话消息
    pub short_term: Vec<Message>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            short_term: Vec::new(),
        }
    }

    /// 添加消息
    pub fn add(&mut self, msg: Message) {
        self.short_term.push(msg);
    }

    /// 获取所有消息
    pub fn all(&self) -> &[Message] {
        &self.short_term
    }

    /// 获取 system 消息
    pub fn system(&self) -> Option<&Message> {
        self.short_term.iter().find(|m| m.role == "system")
    }

    /// 替换 system 消息（用于设置 system prompt）
    pub fn set_system(&mut self, msg: Message) {
        if let Some(pos) = self.short_term.iter().position(|m| m.role == "system") {
            self.short_term[pos] = msg;
        } else {
            self.short_term.insert(0, msg);
        }
    }

    /// 压缩短期记忆：保留 system + 最近 N 条
    pub fn compress(&mut self) {
        if self.short_term.len() <= MAX_SHORT_TERM {
            return;
        }
        let mut system = Vec::new();
        let mut rest = Vec::new();
        for m in self.short_term.drain(..) {
            if m.role == "system" {
                system.push(m);
            } else {
                rest.push(m);
            }
        }
        let keep = rest.split_off(rest.len().saturating_sub(MAX_SHORT_TERM - 1));
        let summary = Message {
            role: "assistant".to_string(),
            content: format!("[上下文已压缩，省略了之前 {} 条消息]", rest.len()),
            tool_call_id: None,
            tool_name: None,
        };
        self.short_term = system;
        self.short_term.push(summary);
        self.short_term.extend(keep);
    }

    /// 轮次数（system 不计）
    pub fn turn_count(&self) -> u32 {
        (self.short_term.iter().filter(|m| m.role == "user").count()) as u32
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
