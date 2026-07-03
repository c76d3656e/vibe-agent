use std::fmt;

/// 运行时错误类型
#[derive(Debug)]
pub enum RuntimeError {
    LlmError(String),
    ToolError(String),
    ParseError(String),
    SessionExpired,
    MaxLoopReached,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::LlmError(msg) => write!(f, "LLM 错误: {}", msg),
            RuntimeError::ToolError(msg) => write!(f, "工具错误: {}", msg),
            RuntimeError::ParseError(msg) => write!(f, "解析错误: {}", msg),
            RuntimeError::SessionExpired => write!(f, "会话已过期"),
            RuntimeError::MaxLoopReached => write!(f, "达到最大循环次数"),
        }
    }
}

impl std::error::Error for RuntimeError {}
