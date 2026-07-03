import React, { useState, useEffect } from 'react'

const KEY_STORAGE = 'vibe_agent_api_key'
const URL_STORAGE = 'vibe_agent_api_url'
const MODEL_STORAGE = 'vibe_agent_api_model'
const DEFAULT_URL = 'https://api.openai.com/v1/chat/completions'
const DEFAULT_MODEL = 'gpt-4o-mini'

export const ApiKeyInput: React.FC = () => {
  const [url, setUrl] = useState(() => localStorage.getItem(URL_STORAGE) || DEFAULT_URL)
  const [model, setModel] = useState(() => localStorage.getItem(MODEL_STORAGE) || DEFAULT_MODEL)
  const [key, setKey] = useState(() => localStorage.getItem(KEY_STORAGE) ?? '')
  const [showKey, setShowKey] = useState(false)
  const [saved, setSaved] = useState(false)
  const hasKey = !!key

  useEffect(() => { localStorage.setItem(KEY_STORAGE, key) }, [key])

  const handleSave = () => {
    localStorage.setItem(KEY_STORAGE, key)
    localStorage.setItem(URL_STORAGE, url)
    localStorage.setItem(MODEL_STORAGE, model)
    setSaved(true)
    setTimeout(() => setSaved(false), 2000)
  }

  const handleClear = () => {
    setKey('')
    setUrl(DEFAULT_URL)
    setModel(DEFAULT_MODEL)
    localStorage.removeItem(KEY_STORAGE)
    localStorage.removeItem(URL_STORAGE)
    localStorage.removeItem(MODEL_STORAGE)
  }

  return (
    <div style={{ ...styles.container, ...(hasKey ? {} : styles.warning) }}>
      {/* API URL */}
      <div style={styles.row}>
        <span style={styles.label}>🔗 URL</span>
        <input
          type="text"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder={DEFAULT_URL}
          style={styles.input}
        />
      </div>

      {/* Model */}
      <div style={{ ...styles.row, marginTop: 5 }}>
        <span style={styles.label}>🧠 Model</span>
        <input
          type="text"
          value={model}
          onChange={(e) => setModel(e.target.value)}
          placeholder={DEFAULT_MODEL}
          style={{ ...styles.input, maxWidth: 200 }}
        />
      </div>

      {/* API Key */}
      <div style={{ ...styles.row, marginTop: 5 }}>
        <span style={styles.label}>
          {hasKey ? '🔑' : '⚠️'} Key
        </span>
        <div style={styles.inputRow}>
          <input
            type={showKey ? 'text' : 'password'}
            value={key}
            onChange={(e) => setKey(e.target.value)}
            placeholder={hasKey ? '••••••••' : 'sk-...'}
            style={{
              ...styles.input,
              flex: 1,
              borderColor: hasKey ? '#0f3460' : '#5c0000',
            }}
          />
          <button style={styles.toggleBtn} onClick={() => setShowKey(!showKey)}>
            {showKey ? '🙈' : '👁️'}
          </button>
        </div>
        <div style={styles.actions}>
          <button style={styles.saveBtn} onClick={handleSave}>
            {saved ? '✅ 已保存' : '保存'}
          </button>
          {hasKey && (
            <button style={styles.clearBtn} onClick={handleClear}>清除</button>
          )}
        </div>
      </div>
    </div>
  )
}

const styles: Record<string, React.CSSProperties> = {
  container: { padding: '6px 14px', background: '#16213e', borderBottom: '1px solid #0f3460' },
  warning: { background: '#2a0000', borderBottom: '1px solid #5c0000' },
  row: { display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' },
  label: { color: '#ccd6f6', fontSize: 12, fontWeight: 600, whiteSpace: 'nowrap', minWidth: 46 },
  inputRow: { display: 'flex', alignItems: 'center', flex: 1, minWidth: 200 },
  input: {
    minWidth: 160,
    padding: '4px 10px',
    borderRadius: 6,
    border: '1px solid #0f3460',
    background: '#0a0a1a',
    color: '#e6f1ff',
    fontSize: 13,
    fontFamily: 'monospace',
    outline: 'none',
  },
  toggleBtn: { background: 'none', border: 'none', cursor: 'pointer', fontSize: 14, padding: '4px 8px' },
  actions: { display: 'flex', gap: 6 },
  saveBtn: { background: '#0f3460', color: '#e6f1ff', border: 'none', padding: '5px 14px', borderRadius: 6, cursor: 'pointer', fontSize: 12 },
  clearBtn: { background: 'transparent', color: '#8892b0', border: '1px solid #333', padding: '5px 14px', borderRadius: 6, cursor: 'pointer', fontSize: 12 },
}
