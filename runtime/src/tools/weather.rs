use async_trait::async_trait;
use super::{ParamDef, Tool};
use serde_json::Value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub struct Weather;

#[async_trait(?Send)]
impl Tool for Weather {
    fn name(&self) -> &'static str { "weather" }
    fn description(&self) -> &'static str { "查询某个城市的实时天气，免费 API 无需密钥。适合查天气、温度、湿度、风向。" }
    fn parameters(&self) -> Vec<ParamDef> {
        vec![ParamDef {
            name: "city",
            param_type: "string",
            description: "城市名称，支持中文和英文，例如：北京、上海、Tokyo",
            required: true,
        }]
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let city = args.get("city").and_then(|v| v.as_str()).ok_or_else(|| "缺少 city 参数".to_string())?;
        let encoded = js_sys::encode_uri_component(city);
        let url = format!("https://uapis.cn/api/v1/misc/weather?city={}&lang=zh", encoded.as_string().unwrap_or_else(|| city.to_string()));

        let window = web_sys::window().ok_or_else(|| "无 window 对象".to_string())?;
        let resp_value = JsFuture::from(window.fetch_with_str(&url)).await
            .map_err(|e| format!("网络请求失败: {:?}", e))?;
        let resp: web_sys::Response = resp_value.dyn_into().map_err(|_| "响应类型错误".to_string())?;

        if !resp.ok() {
            return Err(format!("HTTP {}", resp.status()));
        }

        let text = JsFuture::from(resp.text().map_err(|_| "读取响应失败".to_string())?).await
            .map_err(|e| format!("读取响应失败: {:?}", e))?
            .as_string().ok_or_else(|| "响应为空".to_string())?;

        // 解析 JSON 提取关键信息
        if let Ok(json) = serde_json::from_str::<Value>(&text) {
            let parts = vec![
                json.get("data").and_then(|d| d.get("weather").and_then(|w| w.as_str())).map(|v| format!("天气：{}", v)),
                json.get("data").and_then(|d| d.get("temp").map(|t| format!("温度：{}°C", t))),
                json.get("data").and_then(|d| d.get("humidity").map(|h| format!("湿度：{}%", h))),
                json.get("data").and_then(|d| d.get("wind").map(|w| format!("风：{}", w))),
            ];
            let info: Vec<&str> = parts.iter().filter_map(|p| p.as_deref()).collect();
            if info.is_empty() {
                return Ok(format!("[天气] {} 的天气数据：{}", city, &text.chars().take(200).collect::<String>()));
            }
            return Ok(format!("[天气] {} 今日天气：{}", city, info.join("，")));
        }

        Ok(format!("[天气] {} 的天气数据：{}", city, &text.chars().take(200).collect::<String>()))
    }
}
