/** 由本机 IPv4 推算同 /24 扫描范围 a.b.c.1 ~ a.b.c.254；回环返回 null */
export function inferLanScanRangeFromIpv4(localIp: string): { start: string; end: string } | null {
  const parts = localIp.trim().split('.')
  if (parts.length !== 4) return null
  const o = parts.map((p) => Number(p))
  if (o.some((n) => !Number.isInteger(n) || n < 0 || n > 255)) return null
  if (o[0] === 127) return null
  const [a, b, c] = o
  return {
    start: `${a}.${b}.${c}.1`,
    end: `${a}.${b}.${c}.254`,
  }
}
