pub mod calculator;
pub mod search;
pub mod todo;
pub mod weather;

use async_trait::async_trait;
use serde_json::Value;

/// 工具参数定义
#[derive(Debug, Clone)]
pub struct ParamDef {
    pub name: &'static str,
    pub param_type: &'static str,
    pub description: &'static str,
    pub required: bool,
}

/// 工具 trait — 所有工具必须实现此接口
#[async_trait(?Send)]
pub trait Tool {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters(&self) -> Vec<ParamDef>;
    async fn execute(&self, args: &Value) -> Result<String, String>;
}

/// 工具注册表
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut reg = Self { tools: Vec::new() };
        reg.register(Box::new(calculator::Calculator));
        reg.register(Box::new(search::Search));
        reg.register(Box::new(todo::Todo));
        reg.register(Box::new(weather::Weather));
        reg
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    pub fn has(&self, name: &str) -> bool {
        self.tools.iter().any(|t| t.name() == name)
    }

    pub fn list(&self) -> &[Box<dyn Tool>] {
        &self.tools
    }

    pub fn description(&self) -> String {
        let mut lines = vec!["你有以下工具可以使用：".to_string(), String::new()];
        for tool in &self.tools {
            lines.push(format!("工具名：{}", tool.name()));
            lines.push(format!("用途：{}", tool.description()));
            for p in tool.parameters() {
                let req = if p.required { "（必填）" } else { "（可选）" };
                lines.push(format!("  参数 - {} ({}): {}{}", p.name, p.param_type, p.description, req));
            }
            lines.push(String::new());
        }
        lines.push("调用工具示例：".to_string());
        lines.push(r#"{"action": "use_tool", "tool": "calculator", "params": {"expression": "1+1"}}"#.to_string());
        lines.push(String::new());
        lines.push("直接回答示例：".to_string());
        lines.push(r#"{"action": "answer", "content": "你的回答"}"#.to_string());
        lines.join("\n")
    }

    pub async fn execute(&self, name: &str, args: &Value) -> Result<String, String> {
        match self.tools.iter().find(|t| t.name() == name) {
            Some(tool) => tool.execute(args).await,
            None => Err(format!("没有这个工具：{}，可用工具：{}", name, self.tools.iter().map(|t| t.name()).collect::<Vec<_>>().join(", "))),
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
