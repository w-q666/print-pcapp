#![allow(dead_code)]

use std::net::UdpSocket;

/// 获取本机局域网 IP 地址。
/// 通过创建 UDP socket 连接外部地址来确定本地绑定的网络接口 IP，
/// 不会实际发送任何数据。
pub fn get_local_ip() -> Result<String, String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    socket.connect("8.8.8.8:80").map_err(|e| e.to_string())?;
    let addr = socket.local_addr().map_err(|e| e.to_string())?;
    Ok(addr.ip().to_string())
}

/// 获取所有本地 IPv4 地址（排除 loopback）。
/// MVP 阶段仅返回主 IP；多网卡环境可后续引入 `if-addrs` crate 扩展。
pub fn get_all_local_ips() -> Vec<String> {
    match get_local_ip() {
        Ok(ip) => vec![ip],
        Err(_) => vec!["127.0.0.1".to_string()],
    }
}
