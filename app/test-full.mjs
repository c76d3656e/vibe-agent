/**
 * 完整 E2E 测试 — 覆盖所有工具
 */
import { chromium } from 'playwright'

const URL = 'https://vibe-agent-liard.vercel.app'
const API_URL = 'https://ai.gitee.com/v1'
const API_KEY = '4T81ZFXWCBFFAPBLOSE7IW6PB0ASFVXDTNSA7RES'
const MODEL = 'Qwen3-8B'

async function main() {
  const browser = await chromium.launch({ headless: true })
  const page = await browser.newPage()

  const errors = []
  page.on('pageerror', err => errors.push(err.message))

  // 设置
  await page.goto(URL, { waitUntil: 'networkidle', timeout: 15000 })
  await page.waitForTimeout(2000)
  await page.evaluate(({ url, key, model }) => {
    localStorage.setItem('vibe_agent_api_url', url)
    localStorage.setItem('vibe_agent_api_key', key)
    localStorage.setItem('vibe_agent_api_model', model)
  }, { url: API_URL, key: API_KEY, model: MODEL })

  await page.reload({ waitUntil: 'networkidle', timeout: 15000 })
  await page.waitForTimeout(3000)

  let passed = 0
  let failed = 0

  async function send(msg) {
    try {
      await page.waitForSelector('input[placeholder*="输入"]', { timeout: 5000 })
      await page.fill('input[placeholder*="输入"]', msg)
      await page.waitForTimeout(300)
      await page.click('button:has-text("发送")')
    } catch {
      console.log(`❌ [${msg}] 找不到输入框`); failed++; return
    }
    await page.waitForTimeout(30000)

    if (errors.length > 0) {
      console.log(`❌ [${msg}] 错误:`, errors[errors.length - 1])
      failed++
      errors.length = 0
      return
    }

    const body = await page.textContent('body')
    // 检查错误标记
    if (body.includes('❌')) {
      console.log(`❌ [${msg}] 页面显示错误`)
      failed++
      return
    }

    console.log(`✅ [${msg}] 成功`)
    passed++
  }

  try {
    console.log('\n=== E2E 全工具测试 ===\n')

    // 1. 普通聊天
    await send('hi')

    // 2. 计算器
    await send('计算 1+1 等于多少')

    // 3. 天气
    await send('北京今天天气怎么样')

    // 4. todo 添加
    await send('帮我记下买牛奶')

    // 5. todo 列表
    await send('看看我的待办')

    // 6. 多轮对话
    await send('我刚才让你记了什么？')

    console.log(`\n=== 结果: ${passed} 通过, ${failed} 失败 ===`)
    if (failed > 0) process.exit(1)

  } catch (err) {
    console.error('异常:', err.message)
    process.exit(1)
  } finally {
    await browser.close()
  }
}

main()
