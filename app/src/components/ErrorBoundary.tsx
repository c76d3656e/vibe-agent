import React from 'react'

interface Props {
  children: React.ReactNode
}

interface State {
  error: Error | null
}

export class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = { error: null }
  }

  static getDerivedStateFromError(error: Error) {
    return { error }
  }

  render() {
    if (this.state.error) {
      return (
        <div style={{
          width: '100vw', height: '100vh',
          display: 'flex', flexDirection: 'column',
          alignItems: 'center', justifyContent: 'center',
          background: '#0a0a1a', color: '#ff6b6b',
          padding: 40, fontFamily: 'monospace', fontSize: 14,
        }}>
          <div style={{ fontSize: 48, marginBottom: 16 }}>💥</div>
          <div style={{ fontWeight: 700, marginBottom: 8 }}>Vibe Agent 出错了</div>
          <pre style={{ color: '#ccd6f6', whiteSpace: 'pre-wrap', maxWidth: 600 }}>
            {this.state.error.message}
          </pre>
          <button
            onClick={() => { localStorage.clear(); location.reload() }}
            style={{
              marginTop: 20, padding: '10px 24px',
              background: '#0f3460', color: '#e6f1ff',
              border: 'none', borderRadius: 6, cursor: 'pointer',
            }}
          >
            清除数据并刷新
          </button>
        </div>
      )
    }
    return this.props.children
  }
}
