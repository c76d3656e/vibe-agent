import React from 'react'

interface Props {
  name: string
  args: Record<string, unknown>
  result?: string
}

export const ToolCallCard: React.FC<Props> = ({ name, args, result }) => {
  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <span style={styles.icon}>🔧</span>
        <span style={styles.name}>{name}</span>
      </div>
      <div style={styles.args}>
        <span style={styles.label}>参数:</span>
        <pre style={styles.code}>{JSON.stringify(args, null, 2)}</pre>
      </div>
      {result !== undefined && (
        <div style={styles.result}>
          <span style={styles.label}>结果:</span>
          <pre style={styles.code}>{result}</pre>
        </div>
      )}
    </div>
  )
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    background: '#16213e',
    border: '1px solid #0f3460',
    borderRadius: 8,
    padding: 10,
    margin: '6px 0',
    fontSize: 13,
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    gap: 6,
    marginBottom: 6,
  },
  icon: { fontSize: 14 },
  name: {
    color: '#64ffda',
    fontWeight: 600,
    fontFamily: 'monospace',
  },
  args: { marginBottom: 4 },
  result: {},
  label: {
    color: '#8892b0',
    fontSize: 12,
    display: 'block',
    marginBottom: 2,
  },
  code: {
    margin: 0,
    color: '#ccd6f6',
    fontSize: 12,
    fontFamily: 'monospace',
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-all',
  },
}
