# 系统日志功能完整接入设计

> 日期：2026-05-14
> 状态：已确认，待实施

## 背景

系统日志的完整管道已铺设（SQLite 表 → CRUD repo → Tauri Command → Pinia Store → UI 页面），但**没有任何数据源接入**。`logger.rs` 中的 `log_info/log_warn/log_error` 零调用，前端无任何地方调用 `log_insert`，导致日志页面永远为空。

## 目标

将日志系统从"空壳"变为全覆盖的调试级日志，覆盖：
- HTTP 通信日志（Java 打印服务 + LAN 上传服务）
- 打印流程日志（WebSocket + 任务提交 + 结果）
- 文件操作日志（读/写/删/列表）
- 系统生命周期（启动/关闭/配置变更）
- 全局错误捕获（Vue errorHandler / window.onerror / unhandledrejection）
- 用户关键操作（设置修改、队列清空等）
- 支持 CSV 导出

## 设计决策

| 决策点 | 选择 | 原因 |
|--------|------|------|
| 日志粒度 | 完整调试级（DEBUG/INFO/WARN/ERROR） | 客户端不担心磁盘，全量记录方便排查 |
| 存储 | SQLite system_logs 表（已有） | 支持时间排序、分类筛选、关键词搜索 |
| 导出格式 | CSV | Excel 友好，非技术人员可用 |
| 覆盖范围 | Java 打印服务 + Tauri invoke + LAN HTTP Server | 全覆盖 |
| 自动清理 | 不做 | 用户明确不需要 |
| 实时推送 | 不做 | 手动刷新或进入时 fetch，够用 |
| logger 字段粒度 | 文件+函数级 | 如 `rust:http_server::handle_upload`、`frontend:usePrintService` |

## 日志分类（category）

| category | 含义 | 示例 |
|----------|------|------|
| `system` | 应用生命周期 | 启动、关闭、配置变更 |
| `print` | 打印流程 | 打印提交、成功/失败、WebSocket 状态 |
| `upload` | LAN 上传 | 文件上传请求、token 校验、大小校验 |
| `http` | HTTP 通信 | Java 服务请求/响应、超时、错误 |
| `file` | 文件操作 | 文件读写、删除、列表 |
| `error` | 全局错误 | 未捕获异常、Promise rejection |

## 架构总览

```
┌─────────────────────────────────────────────────────────────┐
│                       日志写入源                              │
├──────────────────────────┬──────────────────────────────────┤
│      前端 (Vue)           │         后端 (Rust)              │
│                          │                                  │
│  useLogger composable    │  logger.rs 工具函数               │
│  ├─ http-client.ts       │  ├─ lib.rs setup (启动/关闭)     │
│  ├─ usePrintService      │  ├─ http_server.rs (LAN上传)     │
│  ├─ useWebSocket         │  ├─ commands.rs (invoke错误)     │
│  ├─ useFileSystem        │  └─ file 操作错误                │
│  ├─ main.ts 全局错误      │                                  │
│  └─ 用户关键操作          │                                  │
│          │               │           │                      │
│     invoke('log_insert') │   SystemLogRepo::insert()        │
│          │               │           │                      │
├──────────┴───────────────┴───────────┴──────────────────────┤
│                    SQLite system_logs 表                      │
├─────────────────────────────────────────────────────────────┤
│                       日志读取                                │
│  SystemLog.vue ← store.fetchLogs() ← invoke('log_query')   │
│  导出 CSV ← invoke('log_export_csv') → Rust 生成文件         │
└─────────────────────────────────────────────────────────────┘
```

## 前端设计

### useLogger composable

```typescript
// src/composables/useLogger.ts
export function useLogger(source: string) {
  return {
    debug(category: string, message: string) { /* 检查 enableDebugLog 开关 */ },
    info(category: string, message: string)  { /* invoke('log_insert', ...) */ },
    warn(category: string, message: string)  { /* invoke('log_insert', ...) */ },
    error(category: string, message: string) { /* invoke('log_insert', ...) */ },
  }
}

// 使用示例：
const logger = useLogger('frontend:http-client::request')
logger.info('http', `GET /getPrintServers → 200 (120ms)`)
logger.error('http', `POST /print/single → 超时 (5000ms)`)
```

- 参数 `source`：声明调用来源，写入 `logger` 字段
- `debug()` 受 settings store 中 `enableDebugLog` 开关控制，关闭时跳过 invoke

### 前端埋点清单

| 模块 | source 标记 | category | 记录内容 |
|------|------------|----------|----------|
| `http-client.ts` | `frontend:http-client::request` | `http` | 每次请求：URL + method + 状态码 + 耗时；失败：错误信息 |
| `usePrintService` | `frontend:usePrintService` | `print` | WS 连接 → 创建任务 → POST 提交 → 结果；每步成功/失败 |
| `useWebSocket` | `frontend:useWebSocket` | `print` | 连接、断开、重连（含次数）、收到状态消息 |
| `useFileSystem` | `frontend:useFileSystem` | `file` | 文件保存/读取/删除：文件名 + 大小 + 结果 |
| `main.ts` | `frontend:global-error-handler` | `error` | Vue errorHandler + window.onerror + unhandledrejection |
| 设置页 | `frontend:settings` | `system` | 修改配置项（旧值 → 新值） |
| 文件管理 | `frontend:file-browser` | `file` | 批量删除、排序切换等 |

### 全局错误处理器（main.ts）

```typescript
// Vue 组件内错误
app.config.errorHandler = (err, instance, info) => {
  logger.error('error', `[Vue] ${err} | 组件: ${instance?.$options.name} | ${info}`)
}

// JS 运行时错误
window.onerror = (message, source, lineno, colno, error) => {
  logger.error('error', `[Runtime] ${message} | ${source}:${lineno}:${colno}`)
}

// 未处理的 Promise rejection
window.addEventListener('unhandledrejection', (event) => {
  logger.error('error', `[Promise] ${event.reason}`)
})
```

## 后端设计

### logger.rs 改造

```rust
pub fn log_info(state: &AppState, category: &str, source: &str, message: &str) {
    let _ = SystemLogRepo::insert(state.db(), "INFO", category, message, source);
}
pub fn log_warn(state: &AppState, category: &str, source: &str, message: &str) { ... }
pub fn log_error(state: &AppState, category: &str, source: &str, message: &str) { ... }
pub fn log_debug(state: &AppState, category: &str, source: &str, message: &str) { ... }
```

签名变更：增加 `source` 参数替代硬编码 `"rust"`，新增 `log_debug`。

### log_insert command 改造

```rust
#[tauri::command]
pub fn log_insert(
    state: tauri::State<'_, AppState>,
    level: String,
    category: String,
    message: String,
    logger: String,  // 新增：来源标记
) -> Result<(), String>
```

### 后端埋点清单

| 位置 | source 标记 | category | 记录内容 |
|------|------------|----------|----------|
| `lib.rs` setup | `rust:lib::setup` | `system` | 应用启动、SQLite 初始化、LAN 服务启动（IP + 端口）、token 生成 |
| `http_server.rs` 上传 | `rust:http_server::handle_upload` | `upload` | 每次上传：IP + 文件名 + 大小 + 结果；token 失败；扩展名拒绝；大小超限 |
| `http_server.rs` 启动 | `rust:http_server::start` | `system` | 服务启动/绑定失败 |
| `commands.rs` 文件操作 | `rust:commands::file_save` 等 | `file` | 路径 + 大小 + 成功/失败 |
| `commands.rs` 打印任务 | `rust:commands::print_jobs_create` 等 | `print` | 任务创建、状态更新、删除 |

### AppState 共享到 axum

当前 `http_server.rs` 无法访问 `AppState`。需要在 `lib.rs` spawn axum 时通过 `Arc<AppState>` 传入，作为 axum 的 Extension state。

### CSV 导出 command

```rust
#[tauri::command]
pub fn log_export_csv(
    state: tauri::State<'_, AppState>,
    path: String,           // 前端通过 dialog.save 获取的文件路径
    level: Option<String>,
    category: Option<String>,
    keyword: Option<String>,
) -> Result<u64, String>    // 返回导出条数
```

- 查询符合筛选条件的所有日志（无 LIMIT）
- CSV 格式：`时间,级别,分类,来源,内容`
- 直接写入用户选择的路径
- 返回导出的记录条数

### 修复已知 bug

`repos.rs` 测试中引用了 `SystemLogRepo::count()` 但该方法未实现，需补充：

```rust
pub fn count(db: &Connection) -> Result<i64, String> {
    db.query_row("SELECT COUNT(*) FROM system_logs", [], |row| row.get(0))
        .map_err(|e| e.to_string())
}
```

## UI 改造

### SystemLog.vue

- **分类标签**：`全部 / system / print / upload / http / file / error`（现有 5 个 → 7 个）
- **级别筛选**：增加 `DEBUG` 选项（现有 INFO / WARN / ERROR）
- **导出按钮**：工具栏"刷新"和"清空"旁边新增"导出 CSV"

### 导出交互流程

1. 用户点击"导出 CSV"
2. 调用 `dialog.save()` 弹出系统文件保存对话框
3. 默认文件名：`系统日志_2026-05-14_180000.csv`
4. 用户选择路径后 → `invoke('log_export_csv', { path, ...当前筛选条件 })`
5. 成功：提示"导出完成，共 N 条记录"
6. 失败：提示错误信息

导出遵循当前筛选条件。

### 设置页新增

在设置页增加日志配置区域：
- **DEBUG 日志开关**：`启用调试级日志`，默认 `false`，持久化到 Tauri Store

## 不做的事情

- 不做日志自动清理 / 大小限制
- 不做日志实时推送（WebSocket 推日志）
- 不做多格式导出（仅 CSV）
- 不做日志分级存储
