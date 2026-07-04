pub mod runtime;
pub mod session;
pub mod context;
pub mod parser;
pub mod trace;
pub mod error;
pub mod tools;
pub mod llm;
pub mod memory;

use wasm_bindgen::prelude::*;
use std::cell::RefCell;

thread_local! {
    static RUNTIME: RefCell<Option<runtime::AgentRuntime>> = RefCell::new(None);
}

/// 启动时的初始化
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        use wasm_bindgen::prelude::*;
        let msg = info.to_string();
        web_sys::console::error_1(&JsValue::from_str(&msg));
    }));
}

#[wasm_bindgen]
pub fn init_runtime(api_url: String, api_key: String, model: String) {
    RUNTIME.with(|rt| {
        *rt.borrow_mut() = Some(runtime::AgentRuntime::new(api_url, api_key, model));
    });
}

#[wasm_bindgen]
pub fn create_session(name: String) -> String {
    RUNTIME.with(|rt| {
        let mut guard = rt.borrow_mut();
        match guard.as_mut() {
            Some(runtime) => {
                let id = runtime.create_session(name);
                match runtime.get_session(&id) {
                    Some(session) => session.to_json(),
                    None => String::new(),
                }
            }
            None => String::new(),
        }
    })
}

#[wasm_bindgen]
pub fn delete_session(id: String) {
    RUNTIME.with(|rt| {
        let mut guard = rt.borrow_mut();
        match guard.as_mut() {
            Some(runtime) => runtime.delete_session(&id),
            None => {}
        }
    });
}

#[wasm_bindgen]
pub fn get_sessions() -> String {
    RUNTIME.with(|rt| {
        let guard = rt.borrow();
        match guard.as_ref() {
            Some(runtime) => {
                let sessions = runtime.list_sessions();
                let json_list: Vec<String> = sessions.iter().map(|s| s.to_json()).collect();
                format!("[{}]", json_list.join(","))
            }
            None => "[]".to_string(),
        }
    })
}

#[wasm_bindgen]
pub fn get_session_messages(session_id: String) -> String {
    RUNTIME.with(|rt| {
        let guard = rt.borrow();
        match guard.as_ref() {
            Some(runtime) => match runtime.get_session(&session_id) {
                Some(session) => serde_json::to_string(&session.memory.short_term).unwrap_or_else(|_| "[]".to_string()),
                None => "[]".to_string(),
            },
            None => "[]".to_string(),
        }
    })
}

#[wasm_bindgen]
pub async fn send_message(session_id: String, input: String) -> String {
    let prep = RUNTIME.with(|rt| {
        let mut guard = rt.borrow_mut();
        match guard.as_mut() {
            Some(runtime) => runtime.prepare_send_message(session_id.clone(), input),
            None => Err("Runtime 未初始化".to_string()),
        }
    });

    let prep = match prep {
        Ok(p) => p,
        Err(e) => {
            return format!(r#"{{"reply":"","traces":[],"error":"{}"}}"#, e);
        }
    };

    let result = run_agent_loop(prep).await;

    RUNTIME.with(|rt| {
        let mut guard = rt.borrow_mut();
        match guard.as_mut() {
            Some(runtime) => runtime.finish_send_message(&session_id, &result),
            None => {}
        }
    });

    serde_json::to_string(&result).unwrap_or_else(|_| r#"{"error":"序列化失败"}"#.to_string())
}

/// Agent 主循环
async fn run_agent_loop(mut prep: runtime::AgentLoopContext) -> runtime::AgentResult {
    let max_loop = 10;

    for _ in 0..max_loop {
        let request_body = llm::build_request(&prep.messages, &prep.model);
        prep.traces.info("LLM 调用".into(), format!("消息数: {}", prep.messages.len()));

        let response_text = match llm::call_llm(&request_body, &prep.api_url, &prep.api_key).await {
            Ok(t) => t,
            Err(e) => {
                let err = format!("LLM 调用失败: {:?}", e);
                prep.traces.error("LLM 错误".into(), err.clone());
                return runtime::AgentResult {
                    reply: err,
                    traces: prep.traces.all().to_vec(),
                    error: Some("LLM call failed".into()),
                    tool_calls: vec![],
                };
            }
        };

        // 提取思考过程并记录日志
        if let Some(think) = parser::extract_think(&response_text) {
            prep.traces.info("🤔 思考过程".into(), think);
        }

        // 解析所有 JSON action
        let actions = parser::parse_actions(&response_text);

        // 没有有效 action → 幻觉，重试
        if actions.is_empty() {
            prep.traces.warn("空响应".into(), "LLM 返回了无有效 JSON 的内容".to_string());
            prep.messages.push(context::Message {
                role: "assistant".into(), content: response_text, tool_call_id: None, tool_name: None,
            });
            prep.messages.push(context::Message {
                role: "user".into(), content: "请输出有效的 JSON。".to_string(), tool_call_id: None, tool_name: None,
            });
            continue;
        }

        // 记录所有解析出的 action
        for a in &actions {
            let thought = a.args.get("__thought__").and_then(|t| t.as_str()).unwrap_or("");
            let desc = if thought.is_empty() {
                format!("{} args: {:?}", a.action, a.args)
            } else {
                format!("{} args: {:?} | thought: {}", a.action, a.args, thought)
            };
            prep.traces.info("action".into(), desc);
        }

        // 检查是否有 final_answer / ask_user
        let has_final = actions.iter().any(|a| a.action == "final_answer" || a.action == "answer");
        let has_ask = actions.iter().any(|a| a.action == "ask_user");
        let final_content = actions.iter()
            .find(|a| a.action == "final_answer" || a.action == "answer")
            .and_then(|a| a.args.get("content").and_then(|c| c.as_str()))
            .unwrap_or("").to_string();
        let ask_content = actions.iter()
            .find(|a| a.action == "ask_user")
            .and_then(|a| a.args.get("question").and_then(|c| c.as_str()))
            .unwrap_or("").to_string();

        // 只有 final_answer/ask_user 无工具调用 → 直接返回
        if has_final && actions.iter().all(|a| a.action == "final_answer" || a.action == "answer") {
            prep.traces.info("💬 最终回答".into(), final_content.clone());
            return runtime::AgentResult { reply: final_content, traces: prep.traces.all().to_vec(), error: None, tool_calls: vec![] };
        }
        if has_ask && actions.iter().all(|a| a.action == "ask_user") {
            prep.traces.info("❓ 向用户提问".into(), ask_content.clone());
            return runtime::AgentResult { reply: format!("[Agent 提问] {}", ask_content), traces: prep.traces.all().to_vec(), error: None, tool_calls: vec![] };
        }

        // 执行所有工具调用
        let mut results_text = String::new();
        for action in &actions {
            if action.action == "final_answer" || action.action == "answer" || action.action == "ask_user" { continue; }
            prep.traces.info("工具调用".into(), format!("{}: {:?}", action.action, action.args));

            if !prep.tool_registry.has(&action.action) {
                results_text += &format!("工具 {} 不存在\n", action.action);
                continue;
            }

            let result = prep.tool_registry.execute(&prep.session_id, &action.action, &serde_json::Value::Object(action.args.clone())).await;
            let result_str = match &result {
                Ok(r) => { prep.traces.info("工具结果".into(), r.clone()); r.clone() }
                Err(e) => { prep.traces.error("工具错误".into(), e.clone()); e.clone() }
            };
            results_text += &format!("[工具结果] {}: {}\n", action.action, result_str);
        }

        // 工具 + final_answer 混合 → 返回最终结果（不回 LLM）
        if has_final {
            prep.traces.info("💬 最终回答".into(), final_content.clone());
            return runtime::AgentResult { reply: final_content, traces: prep.traces.all().to_vec(), error: None, tool_calls: vec![] };
        }

        // 将工具结果发给 LLM 汇总
        prep.messages.push(context::Message {
            role: "assistant".into(), content: response_text, tool_call_id: None, tool_name: None,
        });
        prep.messages.push(context::Message {
            role: "user".into(), content: format!("工具执行结果：\n{}", results_text), tool_call_id: None, tool_name: None,
        });
    }

    prep.traces.warn("循环限制".into(), format!("达到最大循环次数 {}", max_loop));
    runtime::AgentResult {
        reply: "抱歉，处理超时，请重试。".to_string(),
        traces: prep.traces.all().to_vec(),
        error: Some("Max loop reached".into()),
        tool_calls: vec![],
    }
}

#[cfg(test)]
mod tests {
    use crate::session::{self, SessionManager};
    use crate::context::Message;
    use crate::parser;
    use crate::tools::ToolRegistry;
    use crate::trace::TraceLogger;

    // ========== Session ==========

    #[test]
    fn test_session_create() {
        let mut sm = SessionManager::new();
        let s = sm.create("测试".to_string());
        assert!(!s.id.is_empty());
        assert_eq!(s.name, "测试");
        assert_eq!(s.turn_count, 0);
        assert_eq!(s.max_turns, 50);
        assert!(!s.is_expired());
    }

    #[test]
    fn test_session_list_delete() {
        let mut sm = SessionManager::new();
        let id = sm.create("s1".to_string()).id.clone();
        sm.create("s2".to_string());
        assert_eq!(sm.list().len(), 2);
        sm.delete(&id);
        assert_eq!(sm.list().len(), 1);
    }

    #[test]
    fn test_session_expired() {
        let mut sm = SessionManager::new();
        let s = sm.create("exp".to_string());
        s.turn_count = 50;
        assert!(s.is_expired());
        s.turn_count = 0;
        assert!(!s.is_expired());
    }

    // ========== Context 压缩 ==========

    #[test]
    fn test_compress_preserves_system() {
        let mut sm = SessionManager::new();
        let s = sm.create("c".to_string());
        for _ in 0..50 {
            s.memory.short_term.push(Message {
                role: "user".into(), content: "x".into(), tool_call_id: None, tool_name: None,
            });
        }
        s.compress();
        assert_eq!(s.memory.short_term[0].role, "system");
        assert!(s.memory.short_term[1].content.contains("上下文已压缩"));
        assert_eq!(s.memory.short_term.len(), 42);
    }

    #[test]
    fn test_compress_skipped_when_small() {
        let mut sm = SessionManager::new();
        let s = sm.create("c".to_string());
        for _ in 0..10 {
            s.memory.short_term.push(Message {
                role: "user".into(), content: "x".into(), tool_call_id: None, tool_name: None,
            });
        }
        let before = s.memory.short_term.len();
        s.compress();
        assert_eq!(s.memory.short_term.len(), before);
    }

    // ========== Parser ==========

    #[test]
    fn test_parse_answer() {
        let a = parser::parse_actions(r#"{"action": "answer", "content": "你好"}"#);
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].action, "answer");
    }

    #[test]
    fn test_parse_tool_call() {
        let a = parser::parse_actions(r#"{"type": "tool_call", "tool": "weather", "params": {"city": "北京"}}"#);
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].action, "weather");
    }

    #[test]
    fn test_parse_multiple() {
        let raw = r#"{"type": "tool_call", "tool": "weather", "params": {"city": "广州"}}
{"type": "tool_call", "tool": "weather", "params": {"city": "北京"}}"#;
        let a = parser::parse_actions(raw);
        assert_eq!(a.len(), 2);
        assert_eq!(a[0].action, "weather");
        assert_eq!(a[1].action, "weather");
    }

    #[test]
    fn test_parse_think_stripped() {
        let a = parser::parse_actions("<think>思考</think>\n{\"action\": \"answer\", \"content\": \"好\"}");
        assert_eq!(a.len(), 1);
    }

    #[test]
    fn test_parse_empty() {
        assert!(parser::parse_actions("纯文本").is_empty());
    }

    #[test]
    fn test_extract_think() {
        let raw = r#"{"id":"x","choices":[{"index":0,"message":{"role":"assistant","content":"<think>\n思考中\n</think>\n回复"}}]}"#;
        let t = parser::extract_think(raw);
        assert!(t.is_some());
        assert!(t.unwrap().contains("思考中"));
    }

    #[test]
    fn test_extract_text() {
        let raw = r#"{"id":"x","choices":[{"index":0,"message":{"role":"assistant","content":"<think>思考</think>\n你好"}}]}"#;
        let t = parser::extract_text_content(raw);
        assert!(!t.contains("think"));
        assert!(t.contains("你好"));
    }

    // ========== 工具 ==========

    #[test]
    fn test_tools_exist() {
        let reg = ToolRegistry::new();
        assert!(reg.has("calculator"));
        assert!(reg.has("search"));
        assert!(reg.has("todo"));
        assert!(reg.has("weather"));
    }

    #[test]
    fn test_tool_description() {
        let reg = ToolRegistry::new();
        let d = reg.description();
        assert!(d.contains("calculator"));
        assert!(d.contains("工具名"));
    }

    // ========== Trace ==========

    #[test]
    fn test_trace_logger() {
        let mut l = TraceLogger::new();
        l.info("事件".into(), "详情".into());
        l.warn("警告".into(), "注意".into());
        l.error("错误".into(), "失败".into());
        assert_eq!(l.all().len(), 3);
    }

    // ========== System Prompt ==========

    #[test]
    fn test_system_prompt() {
        let p = session::system_prompt();
        assert!(p.contains("tool_call"));
        assert!(p.contains("answer"));
        assert!(p.contains("工具名"));
    }
}
