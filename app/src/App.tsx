import React from 'react'
import { useAgent } from './hooks/useAgent'
import { ApiKeyInput } from './components/ApiKeyInput'
import { SessionTabs } from './components/SessionTabs'
import { ChatView } from './components/ChatView'
import { TracePanel } from './components/TracePanel'

const App: React.FC = () => {
  const {
    sessions,
    currentSessionId,
    messages,
    traces,
    isProcessing,
    loading,
    error,
    createSession,
    deleteSession,
    selectSession,
    sendMessage,
  } = useAgent()

  const currentSession = currentSessionId
    ? { id: currentSessionId, messages }
    : null

  if (loading) {
    return (
      <div style={styles.loading}>
        <div style={styles.loadingIcon}>⚡</div>
        <div style={styles.loadingText}>Vibe Agent</div>
        <div style={styles.loadingSub}>正在加载 WASM Runtime...</div>
      </div>
    )
  }

  return (
    <div style={styles.root}>
      <div style={styles.topBar}>
        <div style={styles.logo}>⚡ Vibe Agent</div>
        <SessionTabs
          sessions={sessions}
          currentSessionId={currentSessionId}
          onSelect={selectSession}
          onCreate={createSession}
          onDelete={deleteSession}
        />
      </div>

      <ApiKeyInput />

      {error && (
        <div style={styles.errorBar}>
          ❌ {error}
        </div>
      )}

      <div style={styles.main}>
        <ChatView
          session={currentSession}
          isProcessing={isProcessing}
          onSend={sendMessage}
        />
        <TracePanel traces={traces} />
      </div>
    </div>
  )
}

const styles: Record<string, React.CSSProperties> = {
  loading: {
    width: '100vw', height: '100vh',
    display: 'flex', flexDirection: 'column',
    alignItems: 'center', justifyContent: 'center',
    background: '#0a0a1a', color: '#64ffda', gap: 8,
  },
  loadingIcon: { fontSize: 48 },
  loadingText: { fontSize: 20, fontWeight: 700, color: '#e6f1ff' },
  loadingSub: { fontSize: 14, color: '#8892b0' },
  root: {
    width: '100vw', height: '100vh',
    display: 'flex', flexDirection: 'column',
    background: '#0a0a1a', color: '#e6f1ff',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    overflow: 'hidden',
  },
  topBar: {
    display: 'flex', alignItems: 'center',
    background: '#0d0d1a', borderBottom: '1px solid #1a1a2e',
  },
  logo: {
    padding: '10px 18px', fontSize: 15, fontWeight: 700,
    color: '#64ffda', whiteSpace: 'nowrap',
    borderRight: '1px solid #1a1a2e',
  },
  errorBar: {
    padding: '6px 14px', background: '#3d0000',
    color: '#ff6b6b', fontSize: 13, borderBottom: '1px solid #5c0000',
  },
  main: {
    flex: 1, display: 'flex', overflow: 'hidden',
  },
}

export default App
