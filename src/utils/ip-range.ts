/** IPv4 扫描范围校验（与 Rust `discovery::expand_scan_range` 规则一致） */

export function parseIpv4(s: string): [number, number, number, number] | null {
  const t = s.trim()
  const parts = t.split('.')
  if (parts.length !== 4) return null
  const nums: number[] = []
  for (const p of parts) {
    if (!/^\d+$/.test(p)) return null
    const v = Number(p)
    if (v > 255 || v < 0) return null
    nums.push(v)
  }
  return [nums[0]!, nums[1]!, nums[2]!, nums[3]!]
}

/** 本机 IPv4 → `a.b.c.1` ~ `a.b.c.254`；回环或非法返回 `null` */
export function inferRangeFromLocalIp(localIp: string): { start: string; end: string } | null {
  const o = parseIpv4(localIp)
  if (!o || o[0] === 127) return null
  const [a, b, c] = o
  return {
    start: `${a}.${b}.${c}.1`,
    end: `${a}.${b}.${c}.254`,
  }
}

export interface ScanRangeValidation {
  ok: boolean
  message?: string
  count?: number
}

/** 两者皆空视为合法（仅探测默认服务 IP） */
export function validateScanRange(start: string, end: string): ScanRangeValidation {
  const st = start.trim()
  const en = end.trim()
  if (!st && !en) return { ok: true, count: 0 }
  if (!st || !en) {
    return { ok: false, message: '扫描起始 IP 与结束 IP 须同时填写或同时留空' }
  }
  const a = parseIpv4(st)
  const b = parseIpv4(en)
  if (!a || !b) return { ok: false, message: '请输入合法的 IPv4 地址' }
  if (a[0] !== b[0] || a[1] !== b[1] || a[2] !== b[2]) {
    return { ok: false, message: '起始与结束 IP 须在同一 /24 网段' }
  }
  if (a[3] > b[3]) {
    return { ok: false, message: '结束 IP 末段须大于等于起始 IP 末段' }
  }
  const count = b[3] - a[3] + 1
  if (count > 300) {
    return { ok: false, message: `扫描范围最多 300 个 IP，当前为 ${count}` }
  }
  return { ok: true, count }
}

export function validateDefaultServiceHost(host: string): ScanRangeValidation {
  const h = host.trim()
  if (!h) return { ok: false, message: '默认服务 IP 不能为空' }
  if (h === 'localhost') return { ok: true }
  if (parseIpv4(h)) return { ok: true }
  return { ok: false, message: '默认服务 IP 须为 localhost 或合法 IPv4' }
}

/** Java 打印服务 HTTP 端口（与探测、getPrintServers 使用同一端口） */
export function validateServicePort(port: number | null | undefined): ScanRangeValidation {
  if (port === null || port === undefined || Number.isNaN(port)) {
    return { ok: false, message: '请输入服务端口' }
  }
  if (!Number.isInteger(port)) {
    return { ok: false, message: '端口须为整数' }
  }
  if (port < 1 || port > 65535) {
    return { ok: false, message: '端口须在 1～65535 之间' }
  }
  return { ok: true }
}
