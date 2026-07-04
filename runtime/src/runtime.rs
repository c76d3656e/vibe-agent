use serde::{Deserialize, Serialize};
use crate::session::{Session, SessionManager};
use crate::context::Message;
use crate::tools::ToolRegistry;
use crate::trace::{TraceEntry, TraceLogger};

/// Agent Runtime 核心
pub struct AgentRuntime {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    session_manager: SessionManager,
}

/// Agent 循环上下文
pub struct AgentLoopContext {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    pub session_id: String,
    pub messages: Vec<Message>,
    pub tool_registry: ToolRegistry,
    pub traces: TraceLogger,
}

/// Agent 循环结果
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResult {
    pub reply: String,
    pub traces: Vec<TraceEntry>,
    pub error: Option<String>,
    pub tool_calls: Vec<String>,
}

impl AgentRuntime {
    pub fn new(api_url: String, api_key: String, model: String) -> Self {
        Self {
            api_url,
            api_key,
            model,
            session_manager: SessionManager::new(),
        }
    }

    // ========== Session 管理 ==========

    pub fn create_session(&mut self, name: String) -> String {
        let session = self.session_manager.create(name);
        session.id.clone()
    }

    pub fn delete_session(&mut self, id: &str) {
        self.session_manager.delete(id);
    }

    pub fn get_session(&self, id: &str) -> Option<&Session> {
        self.session_manager.get(id)
    }

    pub fn list_sessions(&self) -> Vec<&Session> {
        self.session_manager.list()
    }

    // ========== 消息发送准备 ==========

    pub fn prepare_send_message(
        &mut self,
        session_id: String,
        input: String,
    ) -> Result<AgentLoopContext, String> {
        let session = self
            .session_manager
            .get_mut(&session_id)
            .ok_or_else(|| format!("会话不存在: {}", session_id))?;

        if session.is_expired() {
            return Err("会话已过期，请创建新会话".to_string());
        }

        session.memory.short_term.push(Message {
            role: "user".to_string(),
            content: input,
            tool_call_id: None,
            tool_name: None,
        });
        session.turn_count += 1;

        Ok(AgentLoopContext {
            api_url: self.api_url.clone(),
            api_key: self.api_key.clone(),
            model: self.model.clone(),
            session_id: session_id.clone(),
            messages: session.memory.short_term.clone(),
            tool_registry: ToolRegistry::new(),
            traces: TraceLogger::new(),
        })
    }

    pub fn finish_send_message(&mut self, session_id: &str, result: &AgentResult) {
        match self.session_manager.get_mut(session_id) {
            Some(session) => {
                session.memory.short_term.push(Message {
                    role: "assistant".to_string(),
                    content: result.reply.clone(),
                    tool_call_id: None,
                    tool_name: None,
                });
                session.compress();
            }
            None => {}
        }
    }
}
