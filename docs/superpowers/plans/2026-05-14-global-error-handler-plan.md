# 全局异常兜底实施计划

**Goal:** panic 时不崩溃（记录日志），前端未捕获错误降级为 Toast，避免白屏。

**Architecture:** 在 Rust 端设置 panic hook 记录日志；在 Vue 端注册全局 errorHandler 捕获未处理异常并弹出提示。

**Tech Stack:** Rust (std::panic), Vue 3 (app.config.errorHandler)

---

## 文件结构

| 文件 | 操作 |
|------|------|
| `src-tauri/src/lib.rs` | 修改：添加 panic hook |
| `src/main.ts` | 修改：添加 Vue errorHandler |

---

### Task 1: Rust — panic hook

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 在 run() 开头添加 panic hook**

在 `pub fn run()` 函数的最开始（在 `tauri::Builder` 之前）添加：

```rust
pub fn run() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("[PANIC] {}", info);
    }));

    tauri::Builder::default()
    // ...existing code...
```

- [ ] **Step 2: 验证编译**

运行 `cd src-tauri && cargo check`，确认无编译错误。

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add panic hook in lib.rs"
```

---

### Task 2: Vue — 全局 error handler

**Files:**
- Modify: `src/main.ts`

- [ ] **Step 1: 添加 errorHandler**

读取 `src/main.ts`，在 `createApp(App)` 之后、`.use(router)` 之前添加：

```typescript
import { message } from 'ant-design-vue'

const app = createApp(App)

app.config.errorHandler = (err, instance, info) => {
  console.error('[Vue Error]', err, info)
  message.error('操作失败，请稍后重试')
}

app.use(router)
```

- [ ] **Step 2: 验证 TypeScript**

运行 `pnpm vue-tsc --noEmit`，确认无错误。

- [ ] **Step 3: 提交**

```bash
git add src/main.ts
git commit -m "feat: add global Vue errorHandler"
```