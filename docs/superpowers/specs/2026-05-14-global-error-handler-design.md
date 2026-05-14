# 全局异常兜底设计

## 背景

目前 Rust panic 会导致整个应用崩溃退出，前端未捕获异常可能产生白屏或卡死状态，用户体验差。目标是把崩溃变成提示，而非真的"处理掉"错误。

## 设计方案

### 1. Rust — panic hook

在 `lib.rs` 的 `run()` 开头设置 panic hook，记录日志。由于 tauri::async_runtime 使用 tokio 多线程，`set_hook` 只对主线程 panic 生效，tokio 线程 panic 仍会导致进程退出。最完善的保护是 `catch_unwind` 包裹关键 async 块，但本次只做 hook + 日志记录。

```rust
std::panic::set_hook(Box::new(|info| {
    let msg = info.to_string();
    eprintln!("[PANIC] {}", msg);
}));
```

### 2. Vue — 全局 error handler

在 `src/main.ts` 中注册：

```typescript
app.config.errorHandler = (err, instance, info) => {
  console.error('[Vue Error]', err, info)
  message.error('操作失败，请稍后重试')
}
```

## 不做

- 不改动现有各组件的 try-catch 逻辑
- 不包装 `invoke` 调用
- 不做全链路拦截
- 不做 `catch_unwind`（工作量较大）

## 改动文件

| 文件 | 操作 |
|------|------|
| `src-tauri/src/lib.rs` | 修改：添加 panic hook |
| `src/main.ts` | 修改：添加 Vue errorHandler |