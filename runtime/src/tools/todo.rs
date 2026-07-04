use std::cell::RefCell;
use std::collections::HashMap;
use async_trait::async_trait;
use super::{ParamDef, Tool};
use serde_json::Value;

thread_local! {
    static TODOS: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new());
}

pub struct Todo;

#[async_trait(?Send)]
impl Tool for Todo {
    fn name(&self) -> &'static str { "todo" }
    fn description(&self) -> &'static str { "管理待办事项：添加、查看、完成待办任务。" }
    fn parameters(&self) -> Vec<ParamDef> {
        vec![
            ParamDef { name: "cmd", param_type: "string", description: "操作类型：add（添加）, list（查看列表）, done（标记完成）", required: true },
            ParamDef { name: "item", param_type: "string", description: "待办内容（add/done 时需要），例如：'买牛奶'", required: false },
        ]
    }
    async fn execute(&self, session_id: &str, args: &Value) -> Result<String, String> {
        let cmd = args.get("cmd").and_then(|v| v.as_str()).ok_or_else(|| "缺少 cmd 参数".to_string())?;
        match cmd {
            "add" => {
                let item = args.get("item").and_then(|v| v.as_str()).ok_or_else(|| "add 需要 item 参数".to_string())?;
                TODOS.with(|t| t.borrow_mut().entry(session_id.to_string()).or_default().push(item.to_string()));
                Ok(format!("已添加待办: {}", item))
            }
            "list" => {
                Ok(TODOS.with(|t| {
                    let list = t.borrow();
                    let items = list.get(session_id);
                    match items {
                        Some(items) if !items.is_empty() => {
                            let mut s = String::from("待办列表：\n");
                            for (i, item) in items.iter().enumerate() {
                                s.push_str(&format!("{}. {}\n", i + 1, item));
                            }
                            s
                        }
                        _ => "暂无待办事项".to_string(),
                    }
                }))
            }
            "done" => {
                let item = args.get("item").and_then(|v| v.as_str()).ok_or_else(|| "done 需要 item 参数".to_string())?;
                Ok(TODOS.with(|t| {
                    let mut map = t.borrow_mut();
                    let items = map.get_mut(session_id);
                    match items {
                        Some(items) => {
                            let pos = items.iter().position(|x| x == item);
                            match pos {
                                Some(i) => { items.remove(i); format!("已完成: {}", item) }
                                None => format!("未找到待办: {}", item)
                            }
                        }
                        None => format!("未找到待办: {}", item)
                    }
                }))
            }
            _ => Ok(format!("未知命令: {}，支持 add / list / done", cmd)),
        }
    }
}
