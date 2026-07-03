import React from 'react'
import type { TraceEntry } from '../types'

interface Props {
  traces: TraceEntry[]
}

export const TracePanel: React.FC<Props> = ({ traces }) => {
  const bottomRef = React.useRef<HTMLDivElement>(null)

  React.useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [traces.length])

  const levelColors: Record<string, string> = {
    info: '#64ffda',
    warn: '#ffd700',
    error: '#ff6b6b',
  }

  return (
    <div style={styles.container}>
      <div style={styles.header}>📋 执行日志</div>
      <div style={styles.logList}>
        {traces.length === 0 && (
          <div style={styles.empty}>暂无日志</div>
        )}
        {traces.map((t, i) => (
          <div key={i} style={styles.entry}>
            <span
              style={{
                ...styles.level,
                color: levelColors[t.level] ?? '#8892b0',
              }}
            >
              [{t.level.toUpperCase()}]
            </span>
            <span style={styles.time}>
              {new Date(t.timestamp).toLocaleTimeString()}
            </span>
            <div style={styles.details}>
              <span style={styles.event}>{t.event}</span>
              <span style={styles.detailText}>{t.details}</span>
            </div>
          </div>
        ))}
        <div ref={bottomRef} />
      </div>
    </div>
  )
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    background: '#0a0a1a',
    borderLeft: '1px solid #16213e',
    width: 320,
    display: 'flex',
    flexDirection: 'column',
  },
  header: {
    padding: '10px 14px',
    fontSize: 13,
    fontWeight: 600,
    color: '#ccd6f6',
    borderBottom: '1px solid #16213e',
  },
  logList: {
    flex: 1,
    overflowY: 'auto',
    padding: '6px 10px',
    fontSize: 12,
    fontFamily: 'monospace',
  },
  empty: {
    color: '#4a4a6a',
    textAlign: 'center',
    padding: 20,
  },
  entry: {
    marginBottom: 6,
    paddingBottom: 6,
    borderBottom: '1px solid #111128',
  },
  level: {
    fontWeight: 600,
    marginRight: 4,
  },
  time: {
    color: '#4a4a6a',
    fontSize: 11,
    marginRight: 4,
  },
  details: {
    marginTop: 2,
    display: 'flex',
    flexDirection: 'column',
    gap: 2,
  },
  event: {
    color: '#64ffda',
  },
  detailText: {
    color: '#8892b0',
    wordBreak: 'break-all',
  },
}
