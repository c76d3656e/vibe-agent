# Vibe Agent

Rust WASM + React 构建的轻量级 AI Agent，核心 Runtime 自实现，不依赖 LangChain / OpenAI SDK 等框架。

## 一键部署

[![Deploy on Vercel](https://vercel.com/button)](https://vercel.com/new/clone?repository-url=https%3A%2F%2Fgithub.com%2Fc76d3656e%2Fvibe-agent&project-name=vibe-agent&repository-name=vibe-agent&demo-title=Vibe%20Agent&demo-description=Rust%20WASM%20%2B%20React%20%E6%9E%84%E5%BB%BA%E7%9A%84%E8%BD%BB%E9%87%8F%E7%BA%A7%20AI%20Agent&demo-url=https%3A%2F%2Fvibe-agent-liard.vercel.app)

> 点击按钮部署到 Vercel，无需手动配置。  
> API URL / Key / Model 在页面 UI 中设置（存 localStorage），无需环境变量。  
> 默认支持 OpenAI / Gitee AI / Ollama 等任意兼容 /v1/chat/completions 接口的 provider。

## 快速开始

### 环境要求

- Rust 1.70+（wasm32-unknown-unknown target）
- Node.js 18+
- wasm-pack

```bash
# 安装 wasm-pack
cargo install wasm-pack

# 安装前端依赖
cd app && npm install
```

### 构建 WASM

```bash
wasm-pack build runtime --target web --out-dir ../app/wasm-pkg
```

### 开发模式

```bash
cd app && npm run dev
```

### 部署到 Vercel

```bash
cd app
vercel login      # 首次登录
vercel deploy --prod --yes
```

### 运行测试

```bash
# Rust 单元测试（16 项）
cargo test --package vibe-agent-runtime

# E2E 测试（11 项）
cd app && node test-e2e.mjs

# 线上 E2E 测试
cd app && node test-vercel.mjs
```

---

## 系统设计

### 架构图

```
Browser
│
├── React UI（薄壳）
│   ├── ChatView       — 消息展示 + 输入
│   ├── SessionTabs    — 多会话切换
│   ├── TracePanel     — 执行日志
│   └── ApiKeyInput    — API 配置
│
└── WASM Module（@vibe-agent/runtime）
    ├── Agent Runtime     ← Rust 核心循环
    ├── Tool Registry     ← 工具注册 + 调度
    │   ├── Calculator    ← 真实计算（js_sys）
    │   ├── Weather       ← 真实 API（uapis.cn）
    │   ├── Todo          ← 内存存储
    │   └── Search        ← Mock
    ├── Session Manager   ← 多会话隔离
    ├── Context Manager   ← 消息累积 + 压缩
    ├── Output Parser     ← JSON action 解析
    └── Trace Logger      ← 执行链路日志
```

### Agent Loop 流程

```
用户输入
    │
    ▼
prepare_send_message() → 追加 user msg，turn_count++
    │
    ▼
run_agent_loop() (最多 10 轮)
    │
    ├── 1. build_request() → 构造 OpenAI 兼容请求体
    ├── 2. call_llm() → 通过 Vercel proxy (/api/llm) 调 LLM API
    ├── 3. extract_think() → 提取 <think> 内容到 trace
    ├── 4. parse_actions() → 提取所有 {} JSON action
    │       │
    │       ├── 空 → 幻觉重试
    │       ├── answer → 返回结果给用户 ✅
    │       └── use_tool → ↓
    │
    ├── 5. 执行所有工具 (async)
    │       ├── has() 检查工具是否存在
    │       ├── execute() 异步执行
    │       └── 收集全部结果
    │
    └── 6. 工具结果回 LLM → 回到 1
```

### Session 管理

每个 Session 独立，互不影响：

```
Session A (窗口1)        Session B (窗口2)
─────────────────        ─────────────────
id: sess_abc_0012        id: sess_def_0345
messages: [system, ...]  messages: [system, ...]
turn_count: 5            turn_count: 3
max_turns: 50            max_turns: 50
```

- 创建：`createSession()` → WASM 生成唯一 ID
- 切换：`selectSession()` → 从 WASM 读取对应消息
- 删除：`deleteSession()` → WASM 移除 + UI 更新
- 持久化：session 列表存 localStorage（仅用于展示，WASM 状态在内存中）

### Context 管理

**塞入 context 的内容：**

```
[messages 列表]
├── system prompt（工具说明 + 格式规则）
├── user 输入
├── assistant 回复（JSON answer 或 use_tool）
│   └── 工具执行结果（汇总后作为 user 消息回 LLM）
├── user 输入（追问）
├── assistant 回复
│   └── ...
└── ...
```

**最大轮次限制：** `max_turns: 50`，超限返回"会话已过期"。

**上下文压缩（`session.rs:compress()`）：**
- 超 41 条消息时触发
- 保留 system 消息
- 保留最近 40 条（约 20 轮对话）
- 中间插入 `[上下文已压缩]` 标记

### Tool 系统

```rust
pub trait Tool {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters(&self) -> Vec<ParamDef>;
    async fn execute(&self, args: &Value) -> Result<String, String>;
}
```

注册工具只需 `impl Tool` 然后 `registry.register(Box::new(MyTool))`。

当前已注册工具：

| 工具 | 实现 | 说明 |
|------|------|------|
| `calculator` | `js_sys::Function` 求值 | 真实计算 |
| `weather` | `web_sys::fetch` 调 uapis.cn | 真实天气 API |
| `todo` | `thread_local` 内存存储 | 真实增删查 |
| `search` | Mock | 返回模拟结果 |

### Parser（输出解析）

```
LLM 原始响应
    │
    ├── extract_text() → 从 OpenAI 格式中提取 content
    │       │
    │       ├── strip_think_tags() → 去掉 <think> 标签
    │       └── strip_markdown() → 去掉 ```json 标记
    │
    ├── extract_json_objects() → 花括号匹配，提取所有 {}
    │
    └── parse_single_action() → 解析每个 JSON
            │
            ├── use_tool → 拆成 tool name + params
            ├── answer → 提取 content
            └── 旧格式 → 兼容直接 action 名
```

### LLM 调用链路（CORS 处理）

```
Browser WASM → /api/llm (同源) → Vercel Serverless Function → LLM API (Gitee/OpenAI)
```

`/api/llm` 是 Vercel Serverless Function（`api/llm.mjs`），做请求转发，解决浏览器 CORS 限制。

---

## 技术栈

| 层 | 技术 |
|----|------|
| 前端框架 | React 18 + TypeScript |
| 构建工具 | Vite 6 |
| 核心 Runtime | Rust + wasm-bindgen |
| WASM 构建 | wasm-pack |
| 部署 | Vercel（静态 + Serverless Function） |

## 项目结构

```
vibe-agent/
├── runtime/          # Rust WASM 核心
│   ├── src/
│   │   ├── lib.rs        # wasm-bindgen 导出 + Agent 循环
│   │   ├── runtime.rs    # AgentRuntime 结构体
│   │   ├── session.rs    # Session 管理 + Context 压缩
│   │   ├── context.rs    # Message 类型
│   │   ├── llm.rs        # LLM API 调用 + URL 补全
│   │   ├── parser.rs     # JSON action 解析
│   │   ├── trace.rs      # Trace 日志
│   │   ├── error.rs      # 错误类型
│   │   └── tools/        # 工具实现
│   │       ├── mod.rs        # Tool trait + 注册表
│   │       ├── calculator.rs # 计算器
│   │       ├── weather.rs    # 天气
│   │       ├── todo.rs       # 待办
│   │       └── search.rs     # 搜索（Mock）
│   └── Cargo.toml
│
├── app/              # React 前端
│   ├── src/
│   │   ├── wasm.ts        # WASM 桥接
│   │   ├── hooks/useAgent.ts # React ↔ WASM hook
│   │   └── components/    # UI 组件
│   ├── api/llm.mjs      # Vercel Serverless 代理
│   └── package.json
│
└── README.md
```
