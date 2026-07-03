import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import { ErrorBoundary } from './components/ErrorBoundary'

const style = document.createElement('style')
style.textContent = `
  * { margin: 0; padding: 0; box-sizing: border-box; }
  html, body, #root { width: 100%; height: 100%; overflow: hidden; }
  body { background: #0a0a1a; color: #ccd6f6; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
  ::-webkit-scrollbar { width: 6px; }
  ::-webkit-scrollbar-track { background: #0a0a1a; }
  ::-webkit-scrollbar-thumb { background: #333; border-radius: 3px; }
`
document.head.appendChild(style)

const root = document.getElementById('root')
if (root) {
  ReactDOM.createRoot(root).render(
    <ErrorBoundary>
      <App />
    </ErrorBoundary>,
  )
}
