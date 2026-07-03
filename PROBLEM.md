# 问题解决记录

## 1. WASM 环境下 `std::time::SystemTime::now()` 不可用

**问题：** WASM 启动后调用任何函数都抛 `RuntimeError: unreachable`。

**排查：** 通过 `#[wasm_bindgen(start)]` 设置自定义 panic hook 后，控制台输出 `panicked at .../time.rs:35:9: time not implemented on this platform`。

**根因：** `wasm32-unknown-unknown` 目标不支持 `std::time::SystemTime::now()`。`session.rs` 的 `generate_id()` 和 `trace.rs` 的 `now()` 都用了它。

**解决：** 全局搜索所有 `SystemTime::now()` 调用，替换为 `js_sys::Date::now()`。同时做了条件编译以便在非 WASM 环境运行单元测试：

```rust
fn unix_ms() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    { std::time::SystemTime::now()... }
    #[cfg(target_arch = "wasm32")]
    { js_sys::Date::now() as u64 }
}
```

---

## 2. LLM API 跨域（CORS）

**问题：** 浏览器 WASM 直接 `fetch` LLM API 被 CORS 拦截。

**现象：**
```
Access to fetch at 'https://ai.gitee.com/v1' from origin '...' has been blocked by CORS policy
```

**方案对比：**

| 方案 | 说明 | 选择 |
|------|------|------|
| CORS 代理 | 找第三方代理，不可控 | ❌ |
| Vite proxy | 仅开发环境有效 | ❌ |
| **Vercel Serverless Function** | 同源请求 + 转发 | ✅ |

**实现：** `app/api/llm.mjs` 做请求转发。WASM 内 `call_llm()` 构造 `{ url, key, body }` 发给 `/api/llm`，由 Vercel 函数转发到真实 LLM API。

---

## 3. `vite-plugin-wasm` 兼容性问题

**问题：** `--target bundler` + `vite-plugin-wasm` 导致页面空白 / unreachable。

**排查过程：**
- 尝试 `--target bundler` + `vite-plugin-wasm` + `vite-plugin-top-level-await` → unreachable
- 尝试 `--target web` + 动态 `import()` → unreachable
- 尝试 `--target no-modules` → 导入方式不兼容

**根因：** `wasm-bindgen` 生成的 JS 胶水代码中 `import * as wasm from "./...wasm"` 需要 `vite-plugin-wasm` 转换，但插件与 Vite 6 的兼容性有问题，导致 WASM 内部函数（如 `__wbindgen_malloc`）无法访问。

**解决：** 改用 `--target web` + 动态 import，Vite 原生支持 `new URL('...wasm', import.meta.url)` 模式，无需任何额外插件。

```typescript
const mod = await import('vibe-agent-runtime')
await mod.default()  // initWasm — 加载并实例化 WASM
mod.init_runtime(url, key, model)
```

---

## 4. WASM 中字符串切片导致 UTF-8 panic

**问题：** 解析 LLM 返回的中文内容时 panic：
```
byte index 100 is not a char boundary; it is inside '期' (bytes 99..102)
```

**根因：** 错误信息中用了 `&text[..100]` 字节切片，切到了中文字符中间。

**解决：** 全部改为 `text.chars().take(100).collect::<String>()` 按字符截取。

```rust
// 错误
format!("JSON 解析失败: {}", &text[..text.len().min(100)])

// 正确
let preview: String = text.chars().take(100).collect();
format!("JSON 解析失败: {}", preview)
```

---

## 5. API URL 路径不完整导致 "Ready!" 响应

**问题：** 调用 Gitee AI API 始终返回 `Ready!`。

**根因：** 用户配置的 API URL 是 `https://ai.gitee.com/v1`，缺少 `/chat/completions` 路径。`/v1` 根路径返回的是健康检查响应 `Ready!`。

**解决：** 在 `llm.rs` 添加 `normalize_url()` 函数自动补全路径：

```
ai.gitee.com           → https://ai.gitee.com/v1/chat/completions
ai.gitee.com/v1        → https://ai.gitee.com/v1/chat/completions
https://api.openai.com/v1/chat/completions → 不变
```

---

## 6. 全局 processing 锁导致多 Session 阻塞

**问题：** Session1 处理中时，Session2 无法发送消息。

**根因：** 使用 `processingRef = useRef(false)` 全局锁。

**解决：** 改为 `processingSessions: Record<string, boolean>`，每个 session 独立锁：

```typescript
// 之前：所有 session 共用一个锁
processingRef.current = true  // 全阻塞

// 之后：按 session ID 独立锁
processingSessions: { [sessionId]: true }  // 只阻塞当前 session
```

---

## 7. LLM 返回多行 JSON 只解析第一个

**问题：** 用户一次添加 6 个待办，LLM 返回 6 行 JSON，但只执行了第一行。

**根因：** `parse_action()` 只解析第一个 `{}` 对象。

**解决：** 改为 `parse_actions()` 用花括号匹配提取所有顶级 JSON 对象：

```rust
pub fn parse_actions(raw: &str) -> Vec<AgentAction> {
    let jsons = extract_json_objects(&text);  // 提取所有 {}
    for json_str in jsons {
        match parse_single_action(&json_str) {
            Ok(action) => actions.push(action),
            Err(_) => continue,
        }
    }
    actions
}
```

Agent loop 相应改为：收集所有 action → 全部执行 → 汇总结果 → 回 LLM。

---

## 8. `<think>` 标签处理（无关闭标签）

**问题：** LLM 输出 `<think>\n\n{"action": "answer", ...}`（有开始标签无结束标签），`strip_think_tags` 未处理。

**解决：** 无关闭标签时只删开始标签本身，保留内容：

```rust
match result[start..].find(&close) {
    Some(end) => result.drain(start..start + end + close.len()),  // 有关闭 → 删整个块
    None => result.drain(start..start + open.len()),               // 无关闭 → 只删标签
}
```

---

## 9. Session ID 冲突（单元测试发现）

**问题：** 单元测试中在同一毫秒内创建两个 Session，ID 相同导致第二个覆盖第一个。

**根因：** `generate_id()` 只依赖毫秒时间戳 + `ms % 10000`（确定性）。

**解决：** 引入 `AtomicU64` 计数器保证唯一性：

```rust
fn generate_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let ms = unix_ms();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("sess_{:x}_{:04x}", ms, seq % 10000)
}
```

---

## 10. clone 在 async_trait 中导致编译错误

**问题：** `#[async_trait]` 标记的 trait 中，闭包返回 `String` 而非 `Result<String, String>`。

**解决：** 用 `Ok(...)` 包装返回值：

```rust
// 错误
"list" => TODOS.with(|t| { ... })  // 返回 String

// 正确
"list" => Ok(TODOS.with(|t| { ... }))  // 返回 Result<String, String>
```

---

## 11. 多工具协作流程设计

**问题：** LLM 一次请求中需要调多个工具（如查两个城市的天气再计算平均值），但旧的 loop 设计只执行一个工具就回 LLM。

**旧的流程（低效）：**
```
LLM → weather(广州) → 回 LLM → weather(北京) → 回 LLM → calculator → 回 LLM → answer
```

**新的流程（高效）：**
```
LLM → [weather(广州), weather(北京)] → 全部执行 → 汇总 → 回 LLM → calculator → 回 LLM → answer
```

关键改动：`parse_actions()` 支持多 JSON、loop 内遍历执行所有 action。

---

## 12. cdylib 不支持集成测试

**问题：** `cargo test` 在 `tests/` 目录运行集成测试时找不到 crate。

**根因：** `Cargo.toml` 中 `[lib] crate-type = ["cdylib"]`（WASM 专用），不生成 Rust 库文件。

**解决：** 测试改为 `#[cfg(test)]` 模块内联在 `lib.rs` 中：

```rust
#[cfg(test)]
mod tests {
    use crate::session::SessionManager;
    // ... 16 个测试
}
```

---

## 13. Vite / Playwright 测试中输入框定位问题

**问题：** E2E 测试中多次对话后找不到聊天输入框。

**根因：** 页面上有多个 `<input>`（URL / Model / Key / 聊天），`page.locator('input').first()` 选中了 URL 输入框。

**解决：** 用 `placeholder` 区分：

```javascript
// 聊天输入框
page.locator('input[placeholder*="输入"]')
// URL 输入框
page.locator('input[placeholder*="http"]')
```

---

## 附录：关键设计决策

| 决策 | 选项 | 选择理由 |
|------|------|---------|
| WASM 目标 | `--target web` | 无需 bundler 插件，Vite 原生支持 |
| LLM 通信 | JSON action 格式 | 跨平台兼容，不依赖 OpenAI 专有 API |
| Tool 注册 | Rust trait | 类型安全、扩展性 |
| CORS 方案 | Vercel Serverless | 同源请求，生产可用 |
| Context ID | AtomicU64 计数器 | 防止并发 ID 冲突 |
| Parser 策略 | 批量提取所有 JSON | 支持 LLM 一次多 tool call |
