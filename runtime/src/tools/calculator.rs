use async_trait::async_trait;
use super::{ParamDef, Tool};
use serde_json::Value;

pub struct Calculator;

#[async_trait(?Send)]
impl Tool for Calculator {
    fn name(&self) -> &'static str { "calculator" }
    fn description(&self) -> &'static str { "计算数学表达式，支持 + - * / 运算。适合做数学计算、数值运算。" }
    fn parameters(&self) -> Vec<ParamDef> {
        vec![ParamDef {
            name: "expression",
            param_type: "string",
            description: "数学表达式，例如：1 + 2 * 3，支持加减乘除和小括号",
            required: true,
        }]
    }
    async fn execute(&self, _session_id: &str, args: &Value) -> Result<String, String> {
        let expr = args.get("expression").and_then(|v| v.as_str()).ok_or_else(|| "缺少 expression 参数".to_string())?;
        let sanitized: String = expr.chars().filter(|c| c.is_ascii_digit() || "+-*/.() ".contains(*c)).collect();
        let sanitized = sanitized.trim();
        if sanitized.is_empty() { return Err("无效表达式".to_string()); }

        let result = js_sys::Function::new_with_args("expr", &format!("return ({})", sanitized))
            .call1(&wasm_bindgen::JsValue::NULL, &wasm_bindgen::JsValue::from_str(sanitized))
            .map_err(|e| format!("计算错误: {:?}", e))?;

        Ok(format!("计算结果: {}", result.as_f64().ok_or_else(|| "结果不是数字".to_string())?))
    }
}
