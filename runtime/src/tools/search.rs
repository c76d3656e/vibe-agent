use async_trait::async_trait;
use super::{ParamDef, Tool};
use serde_json::Value;

pub struct Search;

#[async_trait(?Send)]
impl Tool for Search {
    fn name(&self) -> &'static str { "search" }
    fn description(&self) -> &'static str { "搜索互联网上的信息。适合查找新闻、事实、最新资讯。" }
    fn parameters(&self) -> Vec<ParamDef> {
        vec![ParamDef {
            name: "query",
            param_type: "string",
            description: "搜索关键词，例如：'Python 教程'",
            required: true,
        }]
    }
    async fn execute(&self, _session_id: &str, args: &Value) -> Result<String, String> {
        let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| "缺少 query 参数".to_string())?;
        Ok(format!(
            "[Mock 搜索结果] 关于 \"{}\" 的模拟结果:\n1. {} 的相关信息\n2. {} 的最新动态\n（此为 Mock 数据）",
            query, query, query
        ))
    }
}
