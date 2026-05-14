# 系统日志功能实现计划

> 基于：`2026-05-14-system-logging-design.md`
> 日期：2026-05-14

## 依赖关系

```
Step 1 (repos.rs) ─┬─→ Step 2 (logger.rs) ─→ Step 5 (lib.rs 埋点)
                   │                        ─→ Step 6 (http_server 埋点)
                   │                        ─→ Step 7 (commands.rs 埋点)
                   ├─→ Step 3 (log_insert command)
                   └─→ Step 4 (log_export_csv command) ─→ Step 9 (注册 commands)
                   
Step 3 ─→ Step 8 (useLogger composable) ─┬─→ Step 10 (http-client 埋点)
                                         ├─→ Step 11 (usePrintService 埋点)
                                         ├─→ Step 12 (useWebSocket 埋点)
                                         ├─→ Step 13 (useFileSystem 埋点)
                                         └─→ Step 14 (main.ts 全局错误)

Step 4 + Step 9 ─→ Step 15 (SystemLog.vue UI)
Step 15 ─→ Step 16 (system-log store)

最后：Step 17 (cargo test 验证)
```

## Step 1: repos.rs — 补充 SystemLogRepo::count() + query_all()

**文件**: `src-tauri/src/repos.rs`

**改动**:
1. 在 `SystemLogRepo impl` 中添加 `count()` 方法：
```rust
pub fn count(db: &Mutex<Connection>) -> Result<i64, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.query_row("SELECT COUNT(*) FROM system_logs", [], |row| row.get(0))
        .map_err(|e| e.to_string())
}
```

2. 添加 `query_all()` 方法（CSV 导出用，无 LIMIT）：
```rust
pub fn query_all(db: &Mutex<Connection>, q: &LogQuery) -> Result<Vec<SystemLog>, String> {
    // 与 query() 相同逻辑，但不加 LIMIT 子句
}
```

**验证**: `cargo test` 通过（修复 test_log_count_and_clear）

---

## Step 2: logger.rs — 改造签名，增加 log_debug

**文件**: `src-tauri/src/logger.rs`

**改动**:
将所有函数签名从 `(state, category, message)` 改为 `(state, category, source, message)`：
```rust
pub fn log_info(state: &AppState, category: &str, source: &str, message: &str) {
    let _ = SystemLogRepo::insert(state.db(), "INFO", category, message, source);
}
pub fn log_warn(state: &AppState, category: &str, source: &str, message: &str) { ... }
pub fn log_error(state: &AppState, category: &str, source: &str, message: &str) { ... }
pub fn log_debug(state: &AppState, category: &str, source: &str, message: &str) { ... }
```

**依赖**: Step 1（repos 基础完好）

---

## Step 3: commands.rs — log_insert 增加 logger 参数

**文件**: `src-tauri/src/commands.rs`

**改动**:
```rust
#[tauri::command]
pub fn log_insert(
    state: tauri::State<'_, AppState>,
    level: String,
    category: String,
    message: String,
    logger: String,  // 新增
) -> Result<(), String> {
    SystemLogRepo::insert(state.db(), &level, &category, &message, &logger)
}
```

**依赖**: Step 1

---

## Step 4: commands.rs — 新增 log_export_csv command

**文件**: `src-tauri/src/commands.rs`

**改动**:
新增 command：
```rust
#[tauri::command]
pub fn log_export_csv(
    state: tauri::State<'_, AppState>,
    path: String,
    level: Option<String>,
    category: Option<String>,
    keyword: Option<String>,
) -> Result<u64, String>
```

逻辑：
1. 调用 `SystemLogRepo::query_all()` 获取全量日志
2. 构造 CSV 内容（BOM + 标题行 + 数据行），字段：时间,级别,分类,来源,内容
3. CSV 字段使用双引号包裹，内部双引号转义
4. 写入 `path` 指定的文件
5. 返回导出条数

**依赖**: Step 1（query_all）

---

## Step 5: lib.rs — setup 阶段添加日志

**文件**: `src-tauri/src/lib.rs`

**改动**:
在 setup 闭包中，各关键步骤后调用 `logger::log_info`：
- SQLite 初始化成功后
- LAN token 生成后
- HTTP 服务 spawn 后

需要在 setup 闭包中通过 `app.state::<AppState>()` 获取 state 引用。

**依赖**: Step 2（logger 新签名）

---

## Step 6: http_server.rs — 添加请求日志

**文件**: `src-tauri/src/http_server.rs`

**改动**:
1. `HttpState` 增加一个 `app_db: Arc<Mutex<Connection>>` 字段（从 lib.rs 传入 AppState 的 db 引用）
2. `lib.rs` 中构造 `HttpState` 时传入 db clone
3. `upload_handler` 中记录日志：
   - 收到上传请求：IP + 文件名 + 大小（INFO, upload）
   - token 校验失败（WARN, upload）
   - 扩展名拒绝（WARN, upload）
   - 文件大小超限（WARN, upload）
   - 上传成功（INFO, upload）
   - 上传失败/写文件失败（ERROR, upload）
4. `start_server` 中记录服务启动成功（INFO, system）

辅助函数：在 `http_server.rs` 中写一个内部 `log_to_db` 函数直接调用 `SystemLogRepo::insert`，避免依赖 `AppState` 包装。

**依赖**: Step 1, Step 2

---

## Step 7: commands.rs — 文件操作和打印任务添加日志

**文件**: `src-tauri/src/commands.rs`

**改动**:
在以下 command 中添加日志调用（通过 `logger::log_xxx`）：

需要给文件相关 command 增加 `state: tauri::State<'_, AppState>` 参数（当前 file_save/read/delete/list 只有 app_handle）。

- `file_save`: 成功记 INFO（文件名+大小），失败记 ERROR
- `file_read`: 成功记 DEBUG（文件名），失败记 ERROR
- `file_delete`: 成功记 INFO（文件名），失败记 ERROR
- `file_list`: 仅 DEBUG 级别
- `print_jobs_create`: 成功记 INFO（任务名+打印机）
- `print_jobs_update_status`: 记 INFO（任务 ID + 新状态）
- `print_jobs_delete`: 记 INFO（任务 ID）

**依赖**: Step 2（logger 新签名）

---

## Step 8: 前端 — 创建 useLogger composable

**文件**: `src/composables/useLogger.ts`（新建）

**内容**:
```typescript
import { invoke } from '@tauri-apps/api/core'
import { useSettings } from '../stores/settings'

export function useLogger(source: string) {
  function writeLog(level: string, category: string, message: string) {
    invoke('log_insert', { level, category, message, logger: source }).catch(console.error)
  }

  return {
    debug(category: string, message: string) {
      const settings = useSettings()
      if (settings.logLevel === 'DEBUG') {
        writeLog('DEBUG', category, message)
      }
    },
    info(category: string, message: string) { writeLog('INFO', category, message) },
    warn(category: string, message: string) { writeLog('WARN', category, message) },
    error(category: string, message: string) { writeLog('ERROR', category, message) },
  }
}
```

注意：`invoke` 调用是 fire-and-forget（`.catch(console.error)`），不阻塞业务逻辑。

settings store 已有 `logLevel` 字段，默认 `'INFO'`，可直接复用作为 DEBUG 开关。

**依赖**: Step 3（log_insert 新签名）

---

## Step 9: lib.rs — 注册新 command

**文件**: `src-tauri/src/lib.rs`

**改动**:
在 `invoke_handler` 中添加 `commands::log_export_csv`。

**依赖**: Step 4

---

## Step 10: http-client.ts — 添加请求日志

**文件**: `src/api/http-client.ts`

**改动**:
在 `request()` 函数中：
1. 记录请求开始时间
2. 成功时记 INFO：`GET /path → 200 (120ms)`
3. 失败时记 ERROR：`GET /path → Error: xxx (120ms)`

```typescript
import { useLogger } from '../composables/useLogger'
const logger = useLogger('frontend:http-client::request')

export async function request<T>(path: string, options: RequestInit = {}): Promise<CommonResult<T>> {
  const method = options.method || 'GET'
  const start = Date.now()
  try {
    const url = `${baseURL}${path}`
    const response = await fetch(url, { ...options, headers: { ...options.headers } })
    const elapsed = Date.now() - start
    if (!response.ok) {
      const errMsg = `${method} ${path} → ${response.status} ${response.statusText} (${elapsed}ms)`
      logger.error('http', errMsg)
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }
    logger.debug('http', `${method} ${path} → ${response.status} (${elapsed}ms)`)
    return response.json()
  } catch (err) {
    const elapsed = Date.now() - start
    logger.error('http', `${method} ${path} → ${(err as Error).message} (${elapsed}ms)`)
    throw err
  }
}
```

**依赖**: Step 8

---

## Step 11: usePrintService.ts — 添加打印流程日志

**文件**: `src/composables/usePrintService.ts`

**改动**:
1. 引入 useLogger
2. `ensureConnected()`: 连接开始 DEBUG、超时 ERROR
3. `print()`: 流程关键步骤记 INFO/ERROR
   - 开始打印（INFO）
   - WS 连接成功（DEBUG）
   - 创建打印任务记录（DEBUG）
   - POST 提交成功（INFO）
   - POST 提交失败（ERROR）

**依赖**: Step 8

---

## Step 12: useWebSocket.ts — 添加连接日志

**文件**: `src/composables/useWebSocket.ts`

**改动**:
1. 引入 useLogger
2. `onOpen`: INFO 连接成功
3. `onMessage`: DEBUG 收到消息
4. `onClose`: WARN 连接断开
5. `onError`: ERROR 连接错误
6. `scheduleReconnect`: DEBUG 重连（第N次，延迟Xms）

**依赖**: Step 8

---

## Step 13: useFileSystem.ts — 添加文件操作日志

**文件**: `src/composables/useFileSystem.ts`

**改动**:
1. 引入 useLogger
2. `saveFile`: INFO 文件保存成功（文件名），ERROR 失败
3. `readFile`: DEBUG 文件读取
4. `deleteFile`: INFO 文件删除，ERROR 失败
5. `listFiles`: DEBUG 文件列表查询

**依赖**: Step 8

---

## Step 14: main.ts — 添加全局错误处理器

**文件**: `src/main.ts`

**改动**:
在 `app.mount('#app')` 之前添加：
```typescript
import { invoke } from '@tauri-apps/api/core'

function logError(message: string) {
  invoke('log_insert', {
    level: 'ERROR',
    category: 'error',
    message,
    logger: 'frontend:global-error-handler'
  }).catch(console.error)
}

app.config.errorHandler = (err, instance, info) => {
  const name = instance?.$options?.name || 'unknown'
  logError(`[Vue] ${err} | component: ${name} | ${info}`)
}

window.onerror = (message, source, lineno, colno) => {
  logError(`[Runtime] ${message} | ${source}:${lineno}:${colno}`)
}

window.addEventListener('unhandledrejection', (event) => {
  logError(`[Promise] ${event.reason}`)
})
```

注意：这里不能用 `useLogger`（因为在 Pinia setup 之前），直接 `invoke`。

**依赖**: Step 3

---

## Step 15: SystemLog.vue — UI 更新

**文件**: `src/views/log/SystemLog.vue`

**改动**:
1. 更新 `categories` 数组：
```typescript
const categories = [
  { key: null, label: '全部', color: '' },
  { key: 'system', label: '系统', color: 'purple' },
  { key: 'print', label: '打印', color: 'green' },
  { key: 'upload', label: '上传', color: 'orange' },
  { key: 'http', label: 'HTTP', color: 'blue' },
  { key: 'file', label: '文件', color: 'cyan' },
  { key: 'error', label: '错误', color: 'red' },
]
```

2. 工具栏添加"导出 CSV"按钮（在"清空"按钮旁边）

3. 添加导出处理函数：
```typescript
import { save } from '@tauri-apps/plugin-dialog'
import { message } from 'ant-design-vue'

async function handleExport() {
  const now = new Date()
  const defaultName = `系统日志_${format(now)}.csv`
  const path = await save({ defaultPath: defaultName, filters: [{ name: 'CSV', extensions: ['csv'] }] })
  if (!path) return
  const count = await invoke<number>('log_export_csv', {
    path,
    level: store.filterLevel || null,
    category: store.filterCategory || null,
    keyword: store.filterKeyword || null,
  })
  message.success(`导出完成，共 ${count} 条记录`)
}
```

4. 日志行格式增加显示 logger 字段

**依赖**: Step 4, Step 9

---

## Step 16: system-log.ts store — 无改动

当前 store 已经完备（fetchLogs / clearLogs / setCategory），无需额外改动。

---

## Step 17: 验证

1. `cd src-tauri && cargo test` — 确保 Rust 测试通过
2. `pnpm build` — 确保前端编译通过
3. `cd src-tauri && cargo check` — 确保 Rust 编译通过

---

## 文件变更汇总

| 文件 | 操作 | Step |
|------|------|------|
| `src-tauri/src/repos.rs` | 修改（添加 count + query_all） | 1 |
| `src-tauri/src/logger.rs` | 修改（改签名 + 增 log_debug） | 2 |
| `src-tauri/src/commands.rs` | 修改（log_insert 签名 + 新增 log_export_csv + 埋点） | 3, 4, 7 |
| `src-tauri/src/lib.rs` | 修改（setup 埋点 + 注册新 command） | 5, 9 |
| `src-tauri/src/http_server.rs` | 修改（添加 db 引用 + 请求日志） | 6 |
| `src/composables/useLogger.ts` | 新建 | 8 |
| `src/api/http-client.ts` | 修改（请求日志） | 10 |
| `src/composables/usePrintService.ts` | 修改（打印流程日志） | 11 |
| `src/composables/useWebSocket.ts` | 修改（连接日志） | 12 |
| `src/composables/useFileSystem.ts` | 修改（文件操作日志） | 13 |
| `src/main.ts` | 修改（全局错误处理器） | 14 |
| `src/views/log/SystemLog.vue` | 修改（分类 + 导出 + logger 显示） | 15 |
