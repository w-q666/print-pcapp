# 局域网 Java 打印服务发现 — 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 应用启动时在 Rust 侧并发探测局域网内 Java 打印服务（`/print/getPrintServers`），命中则更新持久化与前端 `baseURL`；10s 内未命中则保持默认服务 IP 并沿用现有失败提示；系统配置页支持默认 IP 与扫描网段（IPv4、同 /24、≤300）持久化。

**架构:** 探测逻辑集中在 `src-tauri/src/discovery.rs`，经 Tauri `discover_service` / `get_local_ip` 命令暴露；前端 `App.vue` 在加载 store 后编排「补全默认网段 → 扫描 → 写回」；设置页复用校验工具并双 store 保存。`reqwest` 已在项目中；新增 `tokio-util`（`CancellationToken`）与 Tokio `time`（若尚未启用）。

**Tech stack:** Tauri 2、Vue 3、Pinia、Ant Design Vue 4、`reqwest`、`tokio`、`tokio-util`、`serde`/`serde_json`。

**规格依据:** `docs/superpowers/specs/2026-05-15-lan-print-service-discovery-design.md`

---

## 文件映射

| 文件 | 职责 |
|------|------|
| `src-tauri/Cargo.toml` | 增加 `tokio-util`；为 `tokio` 增加 `time` feature（若缺） |
| `src-tauri/src/discovery.rs` | IP 展开与校验、探测主循环、`ScanConfig`/`ScanResult` |
| `src-tauri/src/commands.rs` | `discover_service`、`get_local_ip` 命令封装 |
| `src-tauri/src/lib.rs` | `mod discovery`；`generate_handler` 注册新命令 |
| `src-tauri/capabilities/default.json` | 为新命令声明 capability（若项目按命令列权限） |
| `src/stores/app-config.ts` | `scanStartIp`、`scanEndIp`；读写 store |
| `src/utils/ip-range.ts`（新建） | IPv4 解析、同网段、范围计数、推算 `a.b.c.1`–`a.b.c.254` |
| `src/App.vue` | 启动编排：`setBaseURL`、invoke 扫描、条件 `checkServiceConnection` |
| `src/views/settings/Settings.vue` | 「保存配置」同时 `appConfig.saveToStore()` |
| `src/views/settings/SystemSettingsTab.vue` | 「打印服务」表单与实时校验展示 |
| `src/api/http-client.ts` | 保持 `setBaseURL`；确保启动路径在首次请求前已设置 |

---

### Task 1: Rust — 依赖与模块骨架

**Files:**

- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/discovery.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1:** 在 `Cargo.toml` 添加 `tokio-util`（启用含 `CancellationToken` 的 feature，通常为默认 `sync`）。检查 `tokio` 是否已有 `time`；若无则加入，以支持 `tokio::time::timeout`。
- [ ] **Step 2:** 新建 `discovery.rs`，定义 `ScanConfig`、`ScanResult`（`#[derive(Serialize, Deserialize)]`，`ScanConfig` 使用 `#[serde(rename_all = "camelCase")]` 与前端 `invoke` 载荷一致），占位 `pub async fn discover_print_service(config: ScanConfig) -> Result<ScanResult, String>`。
- [ ] **Step 3:** `lib.rs` 增加 `mod discovery;`，确保 `cargo check` 通过。

**验证:** `cd src-tauri && cargo check`

---

### Task 2: Rust — IP 范围生成与单元测试

**Files:**

- Modify: `src-tauri/src/discovery.rs`

- [ ] **Step 1:** 实现 `parse_ipv4_octets`、`expand_range_24(start, end) -> Result<Vec<String>, String>`：合法 IPv4、前三段一致、`last_end >= last_start`、长度 ≤ 300。
- [ ] **Step 2:** 实现 `infer_lan_scan_range(local_ip: &str) -> Option<(String, String)>`：非 IPv4 或回环则返回 `None`；否则 `a.b.c.1` ~ `a.b.c.254`。
- [ ] **Step 3:** `#[cfg(test)] mod tests`：覆盖 300 边界、301 拒绝、逆序、跨网段、`192.168.1.5` 推算范围。

**验证:** `cd src-tauri && cargo test`（仅跑 discovery 相关模块名亦可）

---

### Task 3: Rust — HTTP 探测与并发取消

**Files:**

- Modify: `src-tauri/src/discovery.rs`

- [ ] **Step 1:** 构建共享 `reqwest::Client`（合理 `connect_timeout`/`timeout` 或每次 `get` 用 `tokio::time::timeout(Duration::from_secs(3), ...)`）。
- [ ] **Step 2:** 实现候选序列：**先**探测 `default_host`（URL 中主机名原样，端口 `config.port`），再探测展开列表（若某 IP 与 default 解析重复则跳过列表中重复项——实现时以「字符串」去重：`default_host` 与 `x.x.x.x` 不等则都探；若 default 已是 IPv4 且在列表中，列表去重）。
- [ ] **Step 3:** 使用 `Semaphore::new(20)` 限制并发；`CancellationToken`：任一成功 `cancel()`；所有子任务在 acquire 后检查 cancelled。
- [ ] **Step 4:** 外层 `tokio::time::timeout(Duration::from_secs(10), inner_future)`；超时或全失败返回 `found_host: None`，填充 `scanned_count` / `elapsed_ms`（可用 `Instant`）。
- [ ] **Step 5:** 解析 JSON：`serde_json::Value` 取 `code`，`code == 0` 为命中；非 200 或解析失败视为未命中。

**验证:** `cargo test` + 本地可选手动：临时对某 IP 起 mock 返回 `{"code":0,"msg":"ok","data":[]}`（可选，YAGNI 可跳过）。

---

### Task 4: Rust — Tauri 命令与权限

**Files:**

- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/capabilities/default.json`

- [ ] **Step 1:** `#[tauri::command] pub async fn discover_service(config: discovery::ScanConfig) -> Result<discovery::ScanResult, String>` 调用 `discovery::discover_print_service`。
- [ ] **Step 2:** `#[tauri::command] pub fn get_local_ip() -> Result<String, String>` 调用 `crate::network::get_local_ip()`（若与模块函数同名冲突，则将模块函数保留为 `get_local_ip`，命令函数命名为 `get_local_ip_address` 等并在前后端统一）。
- [ ] **Step 3:** 在 `lib.rs` 的 `generate_handler!` 中注册上述命令。
- [ ] **Step 4:** 当前 `default.json` 未逐条列出自定义命令（`core:default` 即可）；若后续安全策略要求显式列出，再为 `discover_service` / `get_local_ip` 等追加对应 permission。

**验证:** `cargo check`；`cargo tauri build` 前至少 dev 跑一次（下一 Task 与前端联调）。

---

### Task 5: 前端 — Store 与 IP 工具

**Files:**

- Modify: `src/stores/app-config.ts`
- Create: `src/utils/ip-range.ts`

- [ ] **Step 1:** `app-config` 增加 `scanStartIp`、`scanEndIp`（`ref('')`），`loadFromStore` / `saveToStore` 读写 `scanStartIp`、`scanEndIp`。
- [ ] **Step 2:** `ip-range.ts` 导出：`validateIpv4`、`parseIpv4`、`same24`、`countRange`、`inferRangeFromLocalIp(local: string): { start: string; end: string } | null`（与规格一致：仅 IPv4、非回环时返回 1–254）。
- [ ] **Step 3:** 导出组合校验 `validateScanRange(start, end): { ok: boolean; message?: string; count?: number }`。

**验证:** `pnpm build`（或 `pnpm exec vue-tsc --noEmit`）确保类型通过。

---

### Task 6: 前端 — 启动编排（App.vue）

**Files:**

- Modify: `src/App.vue`
- Modify: `src/api/http-client.ts`（仅当需在别处导入类型或辅助函数时；否则可不改）

- [ ] **Step 1:** `import { invoke } from '@tauri-apps/api/core'`、`import { setBaseURL } from './api/http-client'`。
- [ ] **Step 2:** `onMounted` 顺序：`detect()` → `appConfig.loadFromStore()` → `settings.loadFromStore()`。
- [ ] **Step 3:** 若 `scanStartIp` 与 `scanEndIp` 均为空字符串：调用 `invoke<string>('get_local_ip')`（以实际注册的 Tauri 命令名为准），成功且可推算则写入 `scanStartIp`/`scanEndIp` 并 `appConfig.saveToStore()`；失败则保持空（规格：仅扫默认服务 IP）。
- [ ] **Step 4:** `invoke('discover_service', { config: { defaultHost: appConfig.serviceHost, port: appConfig.servicePort, startIp: 空则 null, endIp: 空则 null } })` —— Rust 侧用 `Option<String>` 接收空网段。
- [ ] **Step 5:** 若返回 `found_host` 有值：赋值 `appConfig.serviceHost`、调用 `setBaseURL(\`http://${host}:${port}\`)`、`saveToStore()`；**不**调用 `getPrintServers`。
- [ ] **Step 6:** 若无命中：不调 `setBaseURL`（保持 store 中的默认 host）；执行现有 `checkServiceConnection()`（仍会 `getPrintServers` 并可能弹窗）。

**验证:** 手测：Java 服务关 → 应警告；开在本机 → 命中 localhost 或局域网 IP 后打印列表/后续功能可用。

---

### Task 7: 前端 — 系统设置 UI 与双 store 保存

**Files:**

- Modify: `src/views/settings/SystemSettingsTab.vue`
- Modify: `src/views/settings/Settings.vue`

- [ ] **Step 1:** `SystemSettingsTab.vue` 顶部增加「打印服务」：`Form` + `Input`（默认服务 IP）+ 只读端口展示 + 两个 IP `Input`（起始/结束）+ `Typography.Text` 或 `Alert` 显示计数与校验结果（Ant Design Vue，禁止原生裸 `input`）。
- [ ] **Step 2:** 使用 `computed` 或 `watch` 调用 `validateScanRange`，非法时保存按钮可 `disabled`（推荐）或在保存时拦截。
- [ ] **Step 3:** `Settings.vue` 的 `handleSave`：`await Promise.all([settings.saveToStore(), appConfig.saveToStore()])`（并处理 loading / 错误消息）。

**验证:** 设置页修改 IP → 保存 → 重启应用 → 值被恢复；非法范围无法保存。

---

### Task 8: 联调与收尾

- [ ] **Step 1:** 全文搜索 `serviceUrl` / `wsUrl` / `getBaseURL` 使用者，确认 WebSocket 与打印队列在扫描成功后使用新 host（通常依赖 `appConfig` 即可）。
- [ ] **Step 2:** `pnpm build` 与 `cd src-tauri && cargo test && cargo check`。
- [ ] **Step 3:** 提交说明清晰的 git commit（可与用户约定是否单 commit 或按 Task 拆分）。

---

## 风险与注意事项

1. **Windows / 防火墙:** 大量 SYN 可能被系统限制；20 并发为规格要求，若现场有问题再降并发（需改规格）。
2. **`localhost` vs `127.0.0.1`:** 默认主机名保持用户配置字符串传入 Rust URL。
3. **Tauri async command:** 确保 `discover_service` 未在内部 `block_in_place` 阻塞运行时；全程 `async` + `spawn` 即可。
4. **命令命名:** 前后端统一 `invoke` 字符串；若 Rust 中 `network::get_local_ip` 与 command 同名冲突，命令可命名为 `get_print_service_local_ip`。

---

## 可选（YAGNI）

- Vitest 覆盖 `ip-range.ts`。
- `wiremock` 集成测试 discovery 全流程。
