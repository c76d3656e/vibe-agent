import React, { useState, useRef, useEffect } from 'react'
import type { Message } from '../types'
import { ToolCallCard } from './ToolCallCard'

interface Props {
  session: { id: string; messages: Message[] } | null
  isProcessing: boolean
  onSend: (message: string) => void
}

/** 解析带 think/reasoning 标签的消息 */
function parseContent(text: string): { think: string; main: string } {
  const tags = ['think', 'reasoning']
  let think = ''
  let main = text

  for (const tag of tags) {
    const open = `<${tag}>`
    const close = `</${tag}>`
    const startIdx = main.indexOf(open)
    const endIdx = main.indexOf(close)
    if (startIdx !== -1 && endIdx !== -1 && endIdx > startIdx) {
      think += main.slice(startIdx + open.length, endIdx).trim() + '\n'
      main = (main.slice(0, startIdx) + main.slice(endIdx + close.length)).trim()
    }
  }

  return { think: think.trim(), main: main.trim() }
}

/** 带折叠 think 的消息气泡 */
const MessageBubble: React.FC<{ role: string; content: string; style: React.CSSProperties }> = ({ role, content, style }) => {
  const { think, main } = parseContent(content)
  const [showThink, setShowThink] = useState(false)

  return (
    <div style={style}>
      {think && (
        <div style={{ marginBottom: main ? 8 : 0 }}>
          <div
            onClick={() => setShowThink(!showThink)}
            style={{
              cursor: 'pointer',
              color: '#64ffda',
              fontSize: 12,
              userSelect: 'none',
              display: 'flex',
              alignItems: 'center',
              gap: 4,
            }}
          >
            <span>{showThink ? '▼' : '▶'}</span>
            <span>推理过程 {showThink ? '点击收起' : '点击展开'}</span>
          </div>
          {showThink && (
            <div style={{
              marginTop: 6,
              padding: 8,
              background: '#0d1117',
              borderRadius: 6,
              color: '#8b949e',
              fontSize: 12,
              fontFamily: 'monospace',
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word',
              lineHeight: 1.5,
            }}>
              {think}
            </div>
          )}
        </div>
      )}
      {main && (
        <div style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
          {main}
        </div>
      )}
    </div>
  )
}

export const ChatView: React.FC<Props> = ({ session, isProcessing, onSend }) => {
  const [input, setInput] = useState('')
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [session?.messages.length])

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    const text = input.trim()
    if (!text || isProcessing) return
    setInput('')
    onSend(text)
  }

  const messageStyles: Record<string, React.CSSProperties> = {
    user: {
      alignSelf: 'flex-end',
      background: '#0f3460',
      color: '#e6f1ff',
      borderRadius: '12px 12px 4px 12px',
      padding: '8px 14px',
      maxWidth: '80%',
      fontSize: 14,
    },
    assistant: {
      alignSelf: 'flex-start',
      background: '#16213e',
      color: '#ccd6f6',
      borderRadius: '12px 12px 12px 4px',
      padding: '8px 14px',
      maxWidth: '80%',
      fontSize: 14,
    },
    tool: {
      alignSelf: 'flex-start',
      background: '#1a1a2e',
      color: '#8892b0',
      borderRadius: 8,
      padding: '6px 12px',
      maxWidth: '80%',
      fontSize: 12,
      fontFamily: 'monospace',
      border: '1px dashed #333',
    },
  }

  return (
    <div style={styles.container}>
      <div style={styles.messageList}>
        {!session && (
          <div style={styles.empty}>
            <div style={styles.emptyIcon}>🤖</div>
            <div style={styles.emptyText}>新建或选择一个会话开始</div>
          </div>
        )}
        {session?.messages.filter(m => m.role !== 'system').map((m, i) => (
          <React.Fragment key={i}>
            {m.tool_name ? (
              <ToolCallCard
                name={m.tool_name}
                args={{}}
                result={m.content}
              />
            ) : (
              <MessageBubble
                role={m.role}
                content={m.content}
                style={messageStyles[m.role as keyof typeof messageStyles] ?? messageStyles.assistant}
              />
            )}
          </React.Fragment>
        ))}
        {isProcessing && (
          <div style={{ ...messageStyles.assistant, opacity: 0.6 }}>
            <span style={{ color: '#64ffda' }}>思考中</span>
            <span style={{ color: '#64ffda' }}>...</span>
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      <form style={styles.inputArea} onSubmit={handleSubmit}>
        <input
          style={styles.input}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={isProcessing ? '正在处理...' : '输入消息...'}
          disabled={isProcessing || !session}
        />
        <button
          type="submit"
          style={{
            ...styles.sendBtn,
            opacity: isProcessing || !input.trim() || !session ? 0.5 : 1,
          }}
          disabled={isProcessing || !input.trim() || !session}
        >
          发送
        </button>
      </form>
    </div>
  )
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    flex: 1, display: 'flex', flexDirection: 'column',
    background: '#0a0a1a',
  },
  messageList: {
    flex: 1, overflowY: 'auto', padding: 16,
    display: 'flex', flexDirection: 'column', gap: 8,
  },
  empty: {
    flex: 1, display: 'flex', flexDirection: 'column',
    alignItems: 'center', justifyContent: 'center', gap: 12,
  },
  emptyIcon: { fontSize: 48 },
  emptyText: { color: '#4a4a6a', fontSize: 14 },
  inputArea: {
    display: 'flex', gap: 8, padding: '10px 14px',
    borderTop: '1px solid #16213e', background: '#1a1a2e',
  },
  input: {
    flex: 1, padding: '10px 14px', borderRadius: 8,
    border: '1px solid #0f3460', background: '#0a0a1a',
    color: '#e6f1ff', fontSize: 14, outline: 'none',
  },
  sendBtn: {
    padding: '10px 20px', borderRadius: 8, border: 'none',
    background: '#0f3460', color: '#e6f1ff',
    cursor: 'pointer', fontSize: 14, fontWeight: 600,
  },
}
