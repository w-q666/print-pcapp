use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanConfig {
    pub default_host: String,
    pub port: u16,
    pub start_ip: Option<String>,
    pub end_ip: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub found_host: Option<String>,
    pub scanned_count: usize,
    pub elapsed_ms: u64,
}

fn parse_ipv4(s: &str) -> Result<[u8; 4], String> {
    let parts: Vec<&str> = s.trim().split('.').collect();
    if parts.len() != 4 {
        return Err(format!("非法 IPv4: {}", s));
    }
    let mut o = [0u8; 4];
    for (i, p) in parts.iter().enumerate() {
        let v: u16 = p
            .parse()
            .map_err(|_| format!("非法 IPv4: {}", s))?;
        if v > 255 {
            return Err(format!("非法 IPv4: {}", s));
        }
        o[i] = v as u8;
    }
    Ok(o)
}

/// 由本机 IPv4 推算同网段扫描范围 `a.b.c.1` ~ `a.b.c.254`；回环则 `None`。
#[allow(dead_code)]
pub fn infer_lan_scan_range(local_ip: &str) -> Option<(String, String)> {
    let o = parse_ipv4(local_ip).ok()?;
    if o[0] == 127 {
        return None;
    }
    Some((
        format!("{}.{}.{}.1", o[0], o[1], o[2]),
        format!("{}.{}.{}.254", o[0], o[1], o[2]),
    ))
}

pub fn expand_scan_range(start: &str, end: &str) -> Result<Vec<String>, String> {
    let a = parse_ipv4(start)?;
    let b = parse_ipv4(end)?;
    if a[0] != b[0] || a[1] != b[1] || a[2] != b[2] {
        return Err("起始与结束 IP 须在同一 /24 网段".to_string());
    }
    if a[3] > b[3] {
        return Err("结束 IP 末段须大于等于起始 IP 末段".to_string());
    }
    let count = (b[3] as usize).saturating_sub(a[3] as usize) + 1;
    if count > 300 {
        return Err(format!("扫描范围最多 300 个 IP，当前为 {}", count));
    }
    let mut out = Vec::with_capacity(count);
    for last in a[3]..=b[3] {
        out.push(format!("{}.{}.{}.{}", a[0], a[1], a[2], last));
    }
    Ok(out)
}

fn ipv4_equal_string(a: &str, b: &str) -> bool {
    match (parse_ipv4(a), parse_ipv4(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => a.trim() == b.trim(),
    }
}

/// 未配置扫描范围时：遍历本机非回环、非链路本地 IPv4 所在 /24，合并去重，最多 300 个地址。
fn auto_ranges_from_interfaces() -> Result<Vec<String>, String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    let interfaces = if_addrs::get_if_addrs().map_err(|e| format!("枚举网卡失败: {}", e))?;
    for iface in interfaces {
        let ip = match iface.addr {
            if_addrs::IfAddr::V4(v4) => v4.ip,
            _ => continue,
        };
        if ip.is_loopback() || ip.is_link_local() {
            continue;
        }
        let o = ip.octets();
        for last in 1u8..=254u8 {
            if out.len() >= 300 {
                return Ok(out);
            }
            let s = format!("{}.{}.{}.{}", o[0], o[1], o[2], last);
            if seen.insert(s.clone()) {
                out.push(s);
            }
        }
    }
    Ok(out)
}

fn build_targets(config: &ScanConfig) -> Result<Vec<String>, String> {
    let dh = config.default_host.trim();
    if dh.is_empty() {
        return Err("默认服务 IP 不能为空".to_string());
    }
    let mut targets = vec![dh.to_string()];

    let st = config
        .start_ip
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let en = config
        .end_ip
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let range_ips: Vec<String> = match (st, en) {
        (None, None) => auto_ranges_from_interfaces()?,
        (Some(s), Some(e)) => expand_scan_range(s, e)?,
        _ => {
            return Err("扫描起始 IP 与结束 IP 须同时填写或同时留空".to_string());
        }
    };

    for ip in range_ips {
        if ipv4_equal_string(&ip, dh) {
            continue;
        }
        targets.push(ip);
    }

    Ok(targets)
}

async fn probe_one(client: &reqwest::Client, host: &str, port: u16) -> bool {
    let url = format!("http://{}:{}/print/getPrintServers", host, port);
    let Ok(resp) = client.get(&url).send().await else {
        return false;
    };
    if !resp.status().is_success() {
        return false;
    }
    let Ok(val): Result<serde_json::Value, _> = resp.json().await else {
        return false;
    };
    match val.get("code") {
        Some(c) if c.as_i64() == Some(0) || c.as_u64() == Some(0) => true,
        _ => false,
    }
}

/// 并发探测上限（提高以缩短全网段扫描时间；仍受单请求超时与总预算约束）
const DISCOVERY_CONCURRENCY: usize = 128;
const PER_PROBE_TIMEOUT_SECS: u64 = 3;

pub async fn discover_print_service(config: ScanConfig) -> Result<ScanResult, String> {
    let instant = Instant::now();
    let scanned_total = Arc::new(AtomicUsize::new(0));
    let targets = build_targets(&config)?;
    let cancel = CancellationToken::new();
    let sem = Arc::new(Semaphore::new(DISCOVERY_CONCURRENCY));
    let port = config.port;

    let n = targets.len().max(1);
    let waves = (n + DISCOVERY_CONCURRENCY - 1) / DISCOVERY_CONCURRENCY;
    let total_timeout_secs = ((waves as u64)
        .saturating_mul(PER_PROBE_TIMEOUT_SECS)
        .saturating_add(25))
    .min(180)
    .max(45);
    let total_timeout = Duration::from_secs(total_timeout_secs);

    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(PER_PROBE_TIMEOUT_SECS))
        .build()
        .map_err(|e| e.to_string())?;

    let cancel_for_timeout = cancel.clone();
    let found_host = tokio::select! {
        h = async {
            let mut set = JoinSet::new();
            for host in targets {
                let c = cancel.clone();
                let sem = sem.clone();
                let client = client.clone();
                let st = scanned_total.clone();
                set.spawn(async move {
                    let Ok(_permit) = sem.acquire_owned().await else {
                        return None;
                    };
                    if c.is_cancelled() {
                        return None;
                    }
                    let ok = probe_one(&client, &host, port).await;
                    st.fetch_add(1, Ordering::Relaxed);
                    if ok {
                        c.cancel();
                        Some(host)
                    } else {
                        None
                    }
                });
            }
            let mut found = None;
            while let Some(res) = set.join_next().await {
                match res {
                    Ok(Some(h)) => {
                        found = Some(h);
                        set.abort_all();
                        break;
                    }
                    Ok(None) | Err(_) => {}
                }
            }
            found
        } => h,
        _ = tokio::time::sleep(total_timeout) => {
            cancel_for_timeout.cancel();
            None
        }
    };

    Ok(ScanResult {
        found_host,
        scanned_count: scanned_total.load(Ordering::Relaxed),
        elapsed_ms: instant.elapsed().as_millis() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_range_ok() {
        let v = expand_scan_range("10.0.0.1", "10.0.0.5").unwrap();
        assert_eq!(v.len(), 5);
        assert_eq!(v[0], "10.0.0.1");
        assert_eq!(v[4], "10.0.0.5");
    }

    #[test]
    fn expand_cross_subnet_err() {
        let r = expand_scan_range("10.0.0.1", "10.0.1.5");
        assert!(r.is_err());
    }

    #[test]
    fn expand_full_octet_ok() {
        let v = expand_scan_range("192.168.1.0", "192.168.1.255").unwrap();
        assert_eq!(v.len(), 256);
    }

    #[test]
    fn expand_reverse_rejected() {
        assert!(expand_scan_range("10.0.0.10", "10.0.0.5").is_err());
    }

    #[test]
    fn infer_skips_loopback() {
        assert!(infer_lan_scan_range("127.0.0.1").is_none());
    }

    #[test]
    fn infer_lan_ok() {
        let (a, b) = infer_lan_scan_range("192.168.3.100").unwrap();
        assert_eq!(a, "192.168.3.1");
        assert_eq!(b, "192.168.3.254");
    }
}
