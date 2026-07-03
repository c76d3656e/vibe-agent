/**
 * WASM 桥接层
 * 使用 wasm-pack --target web 输出
 * 先 init 再通过 exports 对象访问所有函数
 */

let wasm: any = null
let ready = false
let initErr: string | null = null

export async function initRuntime(apiUrl: string, apiKey: string, model: string): Promise<string | null> {
  if (ready) return null
  try {
    const mod = await import('vibe-agent-runtime')
    // --target web 的 default export 是 init 函数
    await mod.default()
    // 初始化后，所有函数挂载在 mod 上
    mod.init_runtime(apiUrl, apiKey, model)
    wasm = mod
    ready = true
    return null
  } catch (e) {
    initErr = String(e)
    return String(e)
  }
}

export function isReady(): boolean { return ready }
export function getInitError(): string | null { return initErr }

function call<T>(name: string, ...args: any[]): T {
  if (!wasm) throw new Error('WASM 未就绪')
  return wasm[name](...args) as T
}

export function createSession(name: string): string { return call('create_session', name) }
export function deleteSession(id: string): void { call('delete_session', id) }
export function getSessions(): string { return call('get_sessions') }
export function getSessionMessages(id: string): string { return call('get_session_messages', id) }
export function sendMessage(sessionId: string, input: string): Promise<string> { return call('send_message', sessionId, input) }
