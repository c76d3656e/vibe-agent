import React from 'react'
import type { Session } from '../types'

interface Props {
  sessions: Session[]
  currentSessionId: string | null
  onSelect: (id: string) => void
  onCreate: () => void
  onDelete: (id: string) => void
}

export const SessionTabs: React.FC<Props> = ({
  sessions,
  currentSessionId,
  onSelect,
  onCreate,
  onDelete,
}) => {
  return (
    <div style={styles.container}>
      <div style={styles.tabs}>
        {sessions.map((s) => (
          <div
            key={s.id}
            style={{
              ...styles.tab,
              ...(s.id === currentSessionId ? styles.tabActive : {}),
            }}
            onClick={() => onSelect(s.id)}
          >
            <span style={styles.tabName}>{s.name}</span>
            <button
              style={styles.closeBtn}
              onClick={(e) => {
                e.stopPropagation()
                onDelete(s.id)
              }}
            >
              ×
            </button>
          </div>
        ))}
      </div>
      <button style={styles.newBtn} onClick={onCreate}>
        + 新建
      </button>
    </div>
  )
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
    padding: '8px 12px',
    background: '#1a1a2e',
    borderBottom: '1px solid #16213e',
  },
  tabs: {
    display: 'flex',
    gap: 4,
    flex: 1,
    overflowX: 'auto',
  },
  tab: {
    display: 'flex',
    alignItems: 'center',
    gap: 6,
    padding: '6px 12px',
    borderRadius: 6,
    cursor: 'pointer',
    background: '#16213e',
    color: '#8892b0',
    fontSize: 13,
    whiteSpace: 'nowrap',
    userSelect: 'none',
  },
  tabActive: {
    background: '#0f3460',
    color: '#e6f1ff',
  },
  tabName: {},
  closeBtn: {
    background: 'none',
    border: 'none',
    color: '#8892b0',
    cursor: 'pointer',
    fontSize: 16,
    padding: '0 2px',
    lineHeight: 1,
  },
  newBtn: {
    background: '#0f3460',
    color: '#e6f1ff',
    border: 'none',
    padding: '6px 14px',
    borderRadius: 6,
    cursor: 'pointer',
    fontSize: 13,
    whiteSpace: 'nowrap',
  },
}
