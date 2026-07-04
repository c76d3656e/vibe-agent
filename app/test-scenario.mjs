/**
 * 复杂多工具调度场景测试：周末出行规划
 *
 * 测试步骤：
 *   ① 查北京天气      → weather 工具
 *   ② 查广州天气      → weather 工具（对比）
 *   ③ 记待办 × 4     → todo 工具批量
 *   ④ 算预算          → calculator 工具
 *   ⑤ 查看待办列表    → todo 工具
 *   ⑥ 搜景点资讯      → search 工具
 *   ⑦ 追加待办        → todo 工具
 *   ⑧ 多轮记忆追问    → 纯对话
 *   ⑨ 最终总结        → 纯对话 + 总结
 */
import { chromium } from 'playwright'

const URL = 'https://vibe-agent-liard.vercel.app'
const API_KEY = '4T81ZFXWCBFFAPBLOSE7IW6PB0ASFVXDTNSA7RES'
const MODEL = 'Qwen3-8B'
const API_URL = 'https://ai.gitee.com/v1'

let passed = 0, failed = 0
async function t(name, fn) { try { await fn(); console.log(`  ✅ ${name}`); passed++ } catch (e) { console.log(`  ❌ ${name}: ${e.message}`); failed++ } }

async function main() {
  console.log('\n=== 复杂多工具场景测试：周末出行规划 ===\n')

  const browser = await chromium.launch({ headless: true })
  const page = await browser.newPage()
  page.setDefaultTimeout(120000)
  const apiCalls = []

  // 拦截 API 请求做校验（设 120s 超时）
  await page.route('**/api/llm', async (route) => {
    const response = await route.fetch({ timeout: 120000 })
    const text = await response.text()
    apiCalls.push({ status: response.status(), body: text, time: Date.now() })
    await route.fulfill({ response })
  })

  await page.goto(URL, { waitUntil: 'networkidle', timeout: 15000 })
  await page.waitForTimeout(2000)
  await page.evaluate(({ url, key, model }) => {
    localStorage.setItem('vibe_agent_api_url', url)
    localStorage.setItem('vibe_agent_api_key', key)
    localStorage.setItem('vibe_agent_api_model', model)
  }, { url: API_URL, key: API_KEY, model: MODEL })
  await page.reload({ waitUntil: 'networkidle', timeout: 15000 })
  await page.waitForTimeout(3000)

  async function send(msg, timeout = 60000) {
    // 等输入框可用
    await page.waitForFunction(() => {
      const inp = document.querySelector('[placeholder*="输入"]')
      return inp && !inp.disabled
    }, { timeout: 30000 })

    await page.fill('[placeholder*="输入"]', msg)
    await page.waitForTimeout(300)
    await page.click('button:has-text("发送")')

    // 等处理完成
    const deadline = Date.now() + timeout
    while (Date.now() < deadline) {
      await new Promise(r => setTimeout(r, 2000))
      const ready = await page.evaluate(() => {
        const inp = document.querySelector('[placeholder*="输入"]')
        return inp && !inp.disabled
      })
      if (ready) {
        await new Promise(r => setTimeout(r, 1000))
        return await page.textContent('body')
      }
      const err = await page.evaluate(() => document.body.textContent?.includes('❌')).catch(() => false)
      if (err) throw new Error('页面错误')
    }
    throw new Error(`超时 ${timeout}ms`)
  }

  // ====== 执行场景 ======

  const before = () => { const n = apiCalls.length; return { count: () => apiCalls.length - n, last: () => apiCalls[apiCalls.length - 1] } }

  await t('① 查北京天气', async () => {
    const c = before()
    const body = await send('这周末去北京玩两天，查一下北京今天的天气')
    if (!body.includes('天气') && !body.includes('温度') && !body.includes('℃'))
      throw new Error('未返回天气信息')
  })

  await t('② 查广州天气', async () => {
    const body = await send('也查一下广州今天的天气')
    if (!body.includes('天气') && !body.includes('温度'))
      throw new Error('未返回广州天气')
  })

  await t('③ 记待办 ×4', async () => {
    await send('帮我记几个待办：1.周六8点出发 2.带身份证相机 3.预订酒店 4.周日晚上回来', 70000)
  })

  await t('④ 算预算', async () => {
    const body = await send('算一下预算：高铁往返260*2，酒店一晚380，餐饮每天150，门票200，总花费多少？', 70000)
  })

  await t('⑤ 查看待办', async () => {
    const body = await send('看看我记了哪些待办', 50000)
    if (body.includes('暂无') || body.includes('没有'))
      console.log('  ⚠️ 待办列表可能为空')
  })

  await t('⑥ 搜景点', async () => {
    await send('搜一下北京故宫开放时间', 50000)
  })

  await t('⑦ 追加待办', async () => {
    await send('再加一个：提前买故宫门票', 50000)
  })

  await t('⑧ 多轮记忆', async () => {
    const body = await send('我之前查过哪些城市的天气？', 50000)
    if (!body.includes('北京') && !body.includes('广州'))
      console.log('  ⚠️ 可能未完全记住城市')
  })

  await t('⑨ 最终总结', async () => {
    const body = await send('总结一下这次北京周末出行的全部计划', 60000)
    if (!body.includes('北京')) throw new Error('总结未提到北京')
  })

  // ====== 统计 API 调用 ======
  console.log(`\n📊 API 调用统计: ${apiCalls.length} 次`)
  let weatherCalls = 0, todoCalls = 0, calcCalls = 0, searchCalls = 0
  for (const call of apiCalls) {
    if (call.body.includes('weather')) weatherCalls++
    if (call.body.includes('todo')) todoCalls++
    if (call.body.includes('calculator')) calcCalls++
    if (call.body.includes('search')) searchCalls++
  }
  console.log(`   weather: ${weatherCalls} 次`)
  console.log(`   todo: ${todoCalls} 次`)
  console.log(`   calculator: ${calcCalls} 次`)
  console.log(`   search: ${searchCalls} 次`)
  console.log(`   HTTP 200: ${apiCalls.filter(c => c.status === 200).length}/${apiCalls.length}`)

  await page.unrouteAll({ behavior: 'ignoreErrors' })
  await browser.close()
  console.log(`\n=== 结果: ${passed} 通过, ${failed} 失败 ===`)
  console.log(`📊 API: ${apiCalls.length} 次调用, HTTP 200: ${apiCalls.filter(c => c.status === 200).length}/${apiCalls.length}`)
}

main()
