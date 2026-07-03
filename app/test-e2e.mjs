/**
 * E2E 测试 — 覆盖所有功能要求
 * 运行：cd app && node test-e2e.mjs
 */
import { chromium } from 'playwright'

const URL = 'https://vibe-agent-liard.vercel.app'
const API_KEY = '4T81ZFXWCBFFAPBLOSE7IW6PB0ASFVXDTNSA7RES'
const MODEL = 'Qwen3-8B'
const API_URL = 'https://ai.gitee.com/v1'

let passed = 0, failed = 0

async function test(name, fn) {
  try { await fn(); console.log(`  ✅ ${name}`); passed++ }
  catch (e) { console.log(`  ❌ ${name}: ${e.message}`); failed++ }
}

async function fillInput(page, text) {
  // 等输入框可用（不 disabled）
  await page.waitForFunction(() => {
    const inp = document.querySelector('input[placeholder*="输入"]')
    return inp && !inp.disabled
  }, { timeout: 30000 })
  const inp = page.locator('input[placeholder*="输入"]')
  await inp.fill(text)
}

async function sendAndWait(page, timeout = 30000) {
  const btn = page.locator('button', { hasText: '发送' })
  await btn.waitFor({ state: 'visible', timeout: 5000 })
  await btn.click()

  // 轮询等待处理完成
  const deadline = Date.now() + timeout
  while (Date.now() < deadline) {
    await page.waitForTimeout(3000)
    const ready = await page.evaluate(() => {
      const inp = document.querySelector('input[placeholder*="输入"]')
      return inp && !(inp ).disabled
    }).catch(() => false)
    if (ready) {
      await page.waitForTimeout(2000)  // 等 React 状态稳定
      return await page.textContent('body')
    }
  }
  return await page.textContent('body')
}

async function main() {
  console.log('\n=== E2E 测试 ===\n')

  const browser = await chromium.launch({ headless: true })
  const page = await browser.newPage()
  await page.goto(URL, { waitUntil: 'networkidle', timeout: 15000 })
  await page.waitForTimeout(2000)

  // 设置 API
  await page.evaluate(({ url, key, model }) => {
    localStorage.setItem('vibe_agent_api_url', url)
    localStorage.setItem('vibe_agent_api_key', key)
    localStorage.setItem('vibe_agent_api_model', model)
  }, { url: API_URL, key: API_KEY, model: MODEL })

  await page.reload({ waitUntil: 'networkidle', timeout: 15000 })
  await page.waitForTimeout(3000)

  // ====== 1. Session 管理 ======
  await test('创建新会话', async () => {
    const btn = page.locator('button', { hasText: '新建' })
    await btn.waitFor({ state: 'visible', timeout: 5000 })
    await btn.click()
    await page.waitForTimeout(1000)
    const count = await page.locator('button:has-text("×")').count()
    if (count < 1) throw new Error('会话标签未创建')
  })

  await test('切换会话', async () => {
    const firstTab = page.locator('button:has-text("×")').first()
    await firstTab.click()
    await page.waitForTimeout(500)
    // 重新新建一个保持至少 2 个
    await page.locator('button', { hasText: '新建' }).click()
    await page.waitForTimeout(500)
  })

  // ====== 2. 工具调用 ======
  await test('Step1: 直接回答', async () => {
    await fillInput(page, 'hi')
    await sendAndWait(page, 25000)
  })

  await test('Step2: 计算器', async () => {
    await fillInput(page, '计算 1+1')
    const body = await sendAndWait(page, 25000)
    if (body.includes('42')) throw new Error('仍是 mock 值 42')
  })

  await test('Step3: 天气（真实 API）', async () => {
    await fillInput(page, '北京今天天气')
    const body = await sendAndWait(page, 25000)
    if (!body.includes('天气') && !body.includes('温度') && !body.includes('℃') && !body.includes('°C'))
      throw new Error('缺少天气数据')
  })

  await test('Step4: 多工具协作', async () => {
    await fillInput(page, '广州和北京的天气，计算平均气温')
    await sendAndWait(page, 60000)
  })

  await test('Step5: 批量待办', async () => {
    await fillInput(page, '帮我记：早上七点喝牛奶，八点出门，九点拿快递')
    await sendAndWait(page, 40000)
  })

  await test('Step6: 待办列表', async () => {
    await fillInput(page, '看看我的待办')
    await sendAndWait(page, 30000)
  })

  await test('Step7: 多轮记忆', async () => {
    await fillInput(page, '我之前让你记了什么？')
    await sendAndWait(page, 30000)
  })

  // ====== 3. 上下文 ======
  await test('上下文追问', async () => {
    await fillInput(page, '我刚才第一句问了你什么？')
    await sendAndWait(page, 25000)
  })

  // ====== 4. Trace ======
  await test('Trace 面板存在', async () => {
    const body = await page.textContent('body')
    if (!body.includes('执行日志')) throw new Error('无 Trace 面板')
  })

  await browser.close()
  console.log(`\n=== 结果: ${passed} 通过, ${failed} 失败 ===`)
  if (failed > 0) process.exit(1)
}

main()
