/**
 * E2E 测试 — 直接在 Vercel 线上跑
 * 使用 Playwright 打开目标页面，捕获所有网络请求
 */
import { chromium } from 'playwright'

const URL = 'https://vibe-agent-liard.vercel.app'
const API_URL = 'https://ai.gitee.com/v1'
const API_KEY = '4T81ZFXWCBFFAPBLOSE7IW6PB0ASFVXDTNSA7RES'
const MODEL = 'Qwen3-8B'

async function main() {
  console.log('打开浏览器...')
  const browser = await chromium.launch({ headless: true })
  const page = await browser.newPage()

  // 拦截 /api/llm 请求
  const apiCalls = []
  await page.route('**/api/llm', async (route) => {
    const postBody = route.request().postData()
    console.log('\n=== /api/llm 请求体 ===')
    console.log(postBody?.substring(0, 600))

    const response = await route.fetch()
    const text = await response.text()
    console.log('响应状态:', response.status())
    console.log('响应体前300:', text.substring(0, 300))
    apiCalls.push({ body: postBody, response: text, status: response.status() })
    await route.fulfill({ response })
  })

  const logs = []
  page.on('console', msg => {
    const t = msg.text()
    if (t.includes('panic') || t.includes('unreachable') || t.includes('Ready') || t.includes('error') || t.includes('Error'))
      logs.push(`[${msg.type()}] ${t}`)
  })
  page.on('pageerror', err => logs.push(`[PAGE_ERROR] ${err.message}`))

  try {
    // 打开页面
    console.log(`\n打开 ${URL}`)
    await page.goto(URL, { waitUntil: 'networkidle', timeout: 15000 })
    await page.waitForTimeout(3000)
    console.log('页面渲染:', (await page.textContent('body')).substring(0, 400))

    // 设置 API 配置
    console.log('\n设置 API 配置...')
    await page.evaluate(({ url, key, model }) => {
      localStorage.setItem('vibe_agent_api_url', url)
      localStorage.setItem('vibe_agent_api_key', key)
      localStorage.setItem('vibe_agent_api_model', model)
    }, { url: API_URL, key: API_KEY, model: MODEL })

    // 刷新
    console.log('刷新页面...')
    await page.reload({ waitUntil: 'networkidle', timeout: 15000 })
    await page.waitForTimeout(3000)
    console.log('刷新后:', (await page.textContent('body')).substring(0, 500))

    // 检查错误
    if (logs.length > 0) {
      console.log('\n错误:', logs.slice(0, 5))
    }

    // 发送消息 "hi"
    console.log('\n发送 "hi"...')
    const input = await page.$('input[placeholder*="输入"]')
    if (!input) { console.log('❌ 找不到输入框'); return }

    await input.fill('hi')
    await page.click('button:has-text("发送")')
    await page.waitForTimeout(20000)

    const body = await page.textContent('body')
    console.log('\n回复后页面:', body.substring(0, 1000))

    // 第二次对话
    console.log('\n\n发送 "1+1=?"...')
    await input.fill('1+1=?')
    await page.click('button:has-text("发送")')
    await page.waitForTimeout(20000)
    const body2 = await page.textContent('body')
    console.log('\n第二次回复后:', body2.substring(0, 1000))

    // 第三次对话：多工具协作
    console.log('\n\n发送 "查询今天广州和北京的天气，计算平均气温"...')
    await input.fill('查询今天广州和北京的天气，计算平均气温')
    await page.click('button:has-text("发送")')
    await page.waitForTimeout(40000)
    const body3 = await page.textContent('body')
    console.log('\n第三次回复后:', body3.substring(0, 1000))

    // 打印 API 结果
    console.log('\n\n=== API 调用汇总 ===')
    for (let i = 0; i < apiCalls.length; i++) {
      const c = apiCalls[i]
      console.log(`\n#${i + 1}: HTTP ${c.status}`)
      try {
        const resp = JSON.parse(c.response)
        const content = resp.choices?.[0]?.message?.content || resp.error || '(无内容)'
        console.log('  内容:', content.substring(0, 200))
      } catch {
        console.log('  原始响应:', c.response?.substring(0, 200))
      }
    }

  } catch (err) {
    console.error('测试失败:', err.message)
  } finally {
    await browser.close()
  }
}

main()
