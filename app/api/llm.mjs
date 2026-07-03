/**
 * Vercel Serverless Function
 * LLM API 代理 — 解决浏览器 CORS 限制
 */
export default async function handler(req, res) {
  // 只接受 POST
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' })
  }

  const { url, key, body } = req.body || {}
  if (!url || !key || !body) {
    return res.status(400).json({ error: 'Missing url, key, or body' })
  }

  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${key}`,
      },
      body: JSON.stringify(body),
    })

    const text = await response.text()

    res.status(response.status).send(text)
  } catch (err) {
    res.status(500).json({ error: err.message })
  }
}
