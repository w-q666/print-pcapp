# 局域网 Java 打印服务发现 — 设计规格

## 背景与目标

客户端依赖局域网内 Java 打印服务（默认端口 `2024`，HTTP 探测使用 `GET /print/getPrintServers`，成功时响应体 `code === 0`）。服务所在主机可能是本机 `localhost`，也可能是同网段其他 IP。应用启动时需要自动探测可用服务并更新连接地址；若在规定时间内未发现，则回退为配置的默认服务 IP（通常为 `localhost`），行为与当前「连接失败提示」一致，不在后台持续轮询或重扫。

## 用户确认的需求要点

| 项目 | 结论 |
|------|------|
| 发现后是否持续监控 | 否；仅启动时扫描一次，下次冷启动再扫 |
| 扫描过程 UI | 静默；不增加全局加载或状态条 |
| 手动重新扫描 | 否；仅重启应用触发 |
| 10s 总超时未找到 | 停止所有探测任务，保持默认服务 IP，与现有一致（如 `notification.warning`） |
| 探测接口 | `GET /print/getPrintServers` |
| 实现层 | Rust（Tauri）+ Tokio；并发语义为 **最多 20 个并发 in-flight 请求**（`Semaphore(20)`），非 OS 线程池字面量 |
| 单请求超时 | 3s |
| 总扫描预算 | 10s（整体 `timeout` 包裹） |
| 找到服务后 | 立即取消其余 in-flight 探测（`CancellationToken`） |
| 设置项 | 默认服务 IP（默认 `localhost`，不必落在扫描范围内）；扫描起始 IP、结束 IPv4；同网段连续；范围内 IP 数 ≤ 300 |
| 首次扫描范围 | 若持久化中起始/结束均为空，则调用 `get_local_ip()`，按本机 IPv4 推算同网段默认范围（如 `a.b.c.1` ~ `a.b.c.254`）并持久化；若取不到本机 IP，则不写入网段范围，仅探测「默认服务 IP」（通常为 `localhost`） |
| 设置页 UI | 系统设置 Tab 顶部新增「打印服务」区块：默认服务 IP、扫描范围两框、只读 `:2024`、底部数量与校验提示；用户已确认线框方案 |

## 架构概览

```text
[启动] Vue App.vue onMounted
  → loadFromStore (serviceHost, servicePort, scanStartIp, scanEndIp)
  → 若 scan 范围为空：invoke get_local_ip + 推算范围 → 写回 store 并 save
  → invoke discover_service(ScanConfig)
  → 若 found：更新 serviceHost + setBaseURL + saveToStore
  → 若未 found：保持原 serviceHost + 现有 checkServiceConnection / warning

[设置页] SystemSettingsTab + app-config store
  → 编辑三项 + 保存时校验（同网段、连续、≤300、合法 IPv4）
  → saveToStore（与现有 settings.json / plugin-store 一致）
```

Rust 新增模块 `discovery.rs`：`discover_service` 命令接收配置，内部 `reqwest` + `tokio::sync::Semaphore(20)` + `tokio_util::sync::CancellationToken` + 外层 `tokio::time::timeout(10s)`。

## 数据与持久化

- **存储**：沿用 `tauri-plugin-store` 的 `settings.json`（与 `app-config` 现有 `serviceHost` / `servicePort` 同文件）。
- **新增键**：`scanStartIp`、`scanEndIp`（字符串）。**首次安装**：两者均为空时，启动流程用本机 IP 推算并写回非空默认值后持久化。**用户编辑**：允许保存合法范围；若用户将范围清空，下一次启动视为「未配置范围」——仅探测默认服务 IP，不再生成网段列表（与「取不到本机 IP」行为一致）。
- **端口**：UI 只读 `2024`；若未来允许改端口，不在本规格范围。

## 扫描算法细节

1. **生成候选列表**：由 `scanStartIp`～`scanEndIp` 解析为 uint32 序列；校验：合法 IPv4、同 /24（前三段相同）、末段 `end >= start`、数量 ≤ 300。
2. **优先级**：将「默认服务 IP」作为**第一个**探测目标（`localhost` 等主机名原样用于 URL）；随后按网段列表顺序探测其余 IP，列表内去重。**300 上限**仅约束「由起始～结束 IP 展开得到的网段地址数量」；默认服务 IP 若不在该展开集合中，额外多 1 次 HTTP 探测，不占用 300 名额。
3. **并发**：每个候选 `spawn` 任务：`semaphore.acquire` → `cancel.is_cancelled()` → `GET http://{host}:{port}/print/getPrintServers` + per-request 3s timeout → 解析 JSON，`code == 0` 视为命中；命中则 `cancel.cancel()` 并返回该 host。
4. **取消与资源**：收到 cancel 后未开始的任务不再发起；已发起请求依赖 client drop 或 let complete（实现选择：cancel 后 acquire 侧退出，已飞请求自然结束）。
5. **总超时**：10s 到则返回 `found_host: None`，不修改调用方传入的默认 host。

## 命令与前端改动清单

**Rust**

- 新增 `src-tauri/src/discovery.rs`。
- `Cargo.toml`：`reqwest`（建议 `rustls-tls` 或项目已有 tls 策略）、`tokio-util`。
- `commands.rs`：`discover_service`、`get_local_ip`（包装 `network::get_local_ip`）。
- `lib.rs`：`mod discovery`；`invoke_handler` 注册上述命令。

**前端**

- `stores/app-config.ts`：`scanStartIp`、`scanEndIp`；`loadFromStore` / `saveToStore` 扩展。
- `App.vue`：挂载流程为「加载配置 → 初始化扫描范围 → `discover_service` → 若命中则更新 `serviceHost` 与 `setBaseURL`」。**不再**在命中后额外调用 `getPrintServers`（避免重复）；若未命中，保留现有 `checkServiceConnection()` 以弹出与今日一致的失败提示。
- `views/settings/SystemSettingsTab.vue`：新增「打印服务」表单区（Ant Design Vue），保存逻辑：若仅系统设置保存，需同时保存 `app-config`（或引导用户在保存时合并两个 store——实现计划明确：设置页「保存配置」应同时 `appConfig.saveToStore()` 与 `settings.saveToStore()`）。
- `http-client.ts`：扫描成功后 `setBaseURL`。
- 校验逻辑：可放 `src/utils/ip-range.ts` 或 composable，供设置页与（可选）Rust 双端一致。

## 错误处理

- 非法范围：设置页保存时 `message.error`，不落盘非法值。
- `discover_service`：内部错误映射为 `Result::Err` 字符串；前端视为未找到或记录日志，不崩溃。
- 本机 IP 获取失败：不写入网段范围，仅探测「默认服务 IP」，并写入系统日志。

## 测试建议

- **Rust**：对 `generate_ip_range`、边界（300 个、301 拒绝）、非同网段、末段逆序做单元测试；对 discovery 可用 `wiremock` 或本地 mock server（可选，计划中评估 YAGNI）。
- **前端**：对 IP 校验纯函数做 Vitest（若项目尚无测试框架则仅在计划中列为可选）。

## 非目标

- 运行中周期性心跳、断线重扫、多网卡枚举（沿用现有 `get_local_ip` 单主 IP）。
- IPv6。
- 修改 Java 服务端。

## 版本与依赖

- Rust 工具链 ≥ 1.88.0（项目既有要求）。
- 遵守仓库规则：设置 UI 使用 Ant Design Vue。

---

本规格经与用户澄清并确认 UI 线框方案；实施前请用户再次审阅本文件。下一步由 `writing-plans` 产出 `docs/superpowers/plans/2026-05-15-lan-print-service-discovery.md` 任务分解。
