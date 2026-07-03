import { useState, useCallback, useEffect, useRef } from 'react'
import type { Session, TraceEntry } from '../types'
import * as Wasm from '../wasm'

interface AgentState {
  sessions: Session[]
  currentSessionId: string | null
  messages: Session['messages']
  traces: TraceEntry[]
  processingSessions: Record<string, boolean>
  loading: boolean
  error: string | null
}

export function useAgent() {
  const [state, setState] = useState<AgentState>({
    sessions: [],
    currentSessionId: null,
    messages: [],
    traces: [],
    processingSessions: {},
    loading: true,
    error: null,
  })
  const initDone = useRef(false)

  // ========== 初始化 ==========

  useEffect(() => {
    if (initDone.current) return
    initDone.current = true

    ;(async () => {
      const apiUrl = localStorage.getItem('vibe_agent_api_url') || 'https://api.openai.com/v1'
      const apiKey = localStorage.getItem('vibe_agent_api_key') || ''
      const model = localStorage.getItem('vibe_agent_api_model') || 'gpt-4o-mini'

      try {
        const err = await Wasm.initRuntime(apiUrl, apiKey, model)
        if (err) { setState((s) => ({ ...s, error: err, loading: false })); return }
      } catch (e) { setState((s) => ({ ...s, error: String(e), loading: false })); return }

      try {
        const sessionJson = Wasm.createSession('会话 1')
        const newSession: Session = JSON.parse(sessionJson)
        const sessionsJson = Wasm.getSessions()
        const sessionsList: Session[] = JSON.parse(sessionsJson)
        localStorage.setItem('sessions', JSON.stringify(sessionsList))
        setState((s) => ({ ...s, sessions: sessionsList, currentSessionId: newSession.id, messages: [], loading: false }))
      } catch (e) {
        setState((s) => ({ ...s, error: `创建会话失败: ${e}`, loading: false }))
      }
    })()
  }, [])

  // ========== 同步 sessions 到 localStorage ==========

  const syncSessions = useCallback((sessions: Session[]) => {
    localStorage.setItem('sessions', JSON.stringify(sessions))
  }, [])

  // ========== 从 WASM 刷新 sessions ==========

  const refreshSessions = useCallback((): Session[] => {
    const json = Wasm.getSessions()
    const sessions: Session[] = JSON.parse(json)
    setState((s) => ({ ...s, sessions }))
    syncSessions(sessions)
    return sessions
  }, [syncSessions])

  // ========== 创建会话 ==========

  const createSession = useCallback(() => {
    if (!Wasm.isReady()) return
    try {
      const sessionJson = Wasm.createSession(`会话 ${state.sessions.length + 1}`)
      const newSession: Session = JSON.parse(sessionJson)
      refreshSessions()
      setState((s) => ({ ...s, currentSessionId: newSession.id, messages: [], traces: [] }))
    } catch (e) {
      setState((s) => ({ ...s, error: String(e) }))
    }
  }, [state.sessions.length, refreshSessions])

  // ========== 删除会话 ==========

  const deleteSession = useCallback((id: string) => {
    if (!Wasm.isReady()) return
    try {
      Wasm.deleteSession(id)
      const sessions = refreshSessions()
      setState((prev) => {
        const nextId = prev.currentSessionId === id ? sessions[0]?.id ?? null : prev.currentSessionId
        return {
          ...prev,
          currentSessionId: nextId,
          messages: nextId ? JSON.parse(Wasm.getSessionMessages(nextId)) : [],
          traces: [],
        }
      })
    } catch (e) {
      setState((prev) => ({ ...prev, error: String(e) }))
    }
  }, [refreshSessions])

  // ========== 切换会话（清除前一个会话的思考状态） ==========

  const selectSession = useCallback((id: string) => {
    const messages = JSON.parse(Wasm.getSessionMessages(id))
    setState((s) => ({ ...s, currentSessionId: id, messages, traces: [] }))
  }, [])

  // ========== 发送消息（按 session 独立处理） ==========

  const sendMessage = useCallback(async (content: string) => {
    const sid = state.currentSessionId
    if (!sid || !Wasm.isReady()) return

    // 只阻止当前 session 的重叠请求，其他 session 不受影响
    setState((s) => {
      if (s.processingSessions[sid]) return s // 已有请求在处理
      return { ...s, processingSessions: { ...s.processingSessions, [sid]: true }, error: null }
    })

    try {
      const resultJson = await Wasm.sendMessage(sid, content)
      const result = JSON.parse(resultJson)

      const sessionsJson = Wasm.getSessions()
      const sessions: Session[] = JSON.parse(sessionsJson)
      syncSessions(sessions)

      setState((s) => {
        // 如果用户切换了 session，不覆盖当前显示的消息
        const isSameSession = s.currentSessionId === sid
        const msgs = isSameSession ? JSON.parse(Wasm.getSessionMessages(sid)) : s.messages
        return {
          ...s, sessions,
          messages: msgs,
          traces: isSameSession ? (result.traces ?? []) : s.traces,
          error: isSameSession ? (result.error ?? null) : s.error,
          processingSessions: { ...s.processingSessions, [sid]: false },
        }
      })
    } catch (err) {
      setState((s) => ({
        ...s,
        error: String(err),
        processingSessions: { ...s.processingSessions, [sid]: false },
      }))
    }
  }, [state.currentSessionId, syncSessions])

  const isProcessing = state.currentSessionId ? !!state.processingSessions[state.currentSessionId] : false

  return {
    sessions: state.sessions,
    currentSessionId: state.currentSessionId,
    messages: state.messages,
    traces: state.traces,
    isProcessing,
    loading: state.loading,
    error: state.error,
    createSession,
    deleteSession,
    selectSession,
    sendMessage,
  }
}
