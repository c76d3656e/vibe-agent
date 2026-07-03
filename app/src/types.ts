/* ========== 消息类型 ========== */

export type MessageRole = 'user' | 'assistant' | 'tool'

export interface Message {
  role: MessageRole
  content: string
  /** 如果是 tool 消息，标记调用的哪个工具 */
  tool_call_id?: string
  tool_name?: string
}

/* ========== Session 类型 ========== */

export interface Session {
  id: string
  name: string
  messages: Message[]
  created_at: number
  updated_at: number
  turn_count: number
  max_turns: number
}

/* ========== 工具类型 ========== */

export interface ToolParamSchema {
  name: string
  type: string
  description: string
  required?: boolean
}

export interface ToolDefinition {
  name: string
  description: string
  parameters: ToolParamSchema[]
}

export interface ToolCall {
  id: string
  name: string
  arguments: Record<string, unknown>
}

export interface ToolResult {
  tool_call_id: string
  name: string
  result: string
}

/* ========== LLM 相关 ========== */

export interface LLMResponse {
  type: 'thinking' | 'tool_call' | 'final_answer'
  content?: string
  tool_calls?: ToolCall[]
}

/* ========== Trace 日志 ========== */

export interface TraceEntry {
  timestamp: number
  event: string
  details: string
  level: 'info' | 'warn' | 'error'
}

/* ========== Runtime 状态 ========== */

export interface RuntimeState {
  sessions: Session[]
  current_session_id: string | null
  traces: TraceEntry[]
  is_processing: boolean
}
