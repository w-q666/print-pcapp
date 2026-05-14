# 云打印客户端 — 总体开发计划

**日期:** 2026-05-14
**项目:** Tauri 2 + Vue 3 + antd 6.x 跨平台打印客户端
**目标:** 对接已部署的 Java Print Service，实现桌面端 + 手机扫码上传的完整打印体验

---

## 计划索引

| Plan | 名称 | 文件 | 依赖 |
|------|------|------|------|
| 1 | 前端基础设施与布局 | `plan1-frontend-infrastructure.md` | 无 |
| 2 | Rust 后端 Schema 扩展与日志 | `plan2-rust-schema-extension.md` | 无 |
| 3 | Java Print Service 对接 | `plan3-java-service-integration.md` | Plan 1 |
| 4 | 文件管理与预览 | `plan4-file-management-preview.md` | Plan 1 |
| 5 | LAN HTTP 服务与手机扫码 | `plan5-lan-http-server.md` | 无 |
| 6 | 配置/日志/历史页面 | `plan6-settings-log-history-ui.md` | Plan 1, 2 |
| 7 | 打印执行与端到端集成 | `plan7-print-execution-integration.md` | 全部 |

---

## 依赖图

```
第一波（并行启动）:
  Plan 1 ─── 前端基础（Router/Pinia/Layout）
  Plan 2 ─── Rust 后端（Schema/日志/迁移）
  Plan 5 ─── LAN HTTP（axum/QR码）

第二波（等 Plan 1 完成后并行）:
  Plan 3 ─── Java 服务对接（API/WebSocket/打印编排）
  Plan 4 ─── 文件管理（上传/预览）
  Plan 6 ─── 配置/日志/历史页面（等 Plan 2 也完成）

第三波（等全部完成）:
  Plan 7 ─── 打印执行与集成（PrintDialog/StatusOverlay/HomePage）
```

---

## 执行建议

### 多 Agent 并行策略

1. **Agent A** → Plan 1（前端基础） → Plan 3（Java 对接）
2. **Agent B** → Plan 2（Rust Schema） → Plan 6（页面 UI）
3. **Agent C** → Plan 5（LAN HTTP） → Plan 4（文件管理）
4. **Agent D** → Plan 7（集成，等前面全部完成）

### 分支管理建议

每个 Plan 在独立 feature 分支上开发，完成后合并到 main：

```
main
├── feat/plan1-frontend-infra
├── feat/plan2-rust-schema
├── feat/plan3-java-integration
├── feat/plan4-file-management
├── feat/plan5-lan-http
├── feat/plan6-settings-log-history
└── feat/plan7-print-integration
```

Plan 1、2、5 无冲突可直接并行。Plan 3/4/6 需要在 Plan 1 合并后才能开始。

### 冲突风险点

| 文件 | 涉及 Plan | 风险 | 缓解 |
|------|----------|------|------|
| `src-tauri/src/lib.rs` | 2, 5 | 同时修改 command 注册列表 | 约定追加位置，先合并完再续 |
| `src-tauri/src/commands.rs` | 2, 5 | 同时添加新 commands | 分区域写，减少行级冲突 |
| `src/router/index.ts` | 3, 4, 6 | 添加不同路由 | 合并后统一调整路由顺序 |
| `src/stores/index.ts` | 3, 4, 6 | 导出新 stores | 追加式修改，冲突小 |

---

## 当前代码基线

已完成（可复用）：
- Tauri 2 + Vue 3 脚手架
- SQLite: `print_jobs` 表（基础字段）+ CRUD
- 文件管理: `file_save/read/delete/list` commands
- Plugins: `plugin-store`, `plugin-dialog`, `plugin-opener`
- antd 6.x 已安装

待清理：
- `src/App.vue` 中的脚手架 demo 代码（Plan 1 会替换）

---

## 外部依赖

| 依赖 | 版本/地址 | 用途 |
|------|----------|------|
| Java Print Service | `http://localhost:2024` | 打印机枚举、打印执行、状态推送 |
| Java Print WebSocket | `ws://localhost:2024/print` | 打印状态实时推送 |
| LibreOffice (可选) | 系统安装 | Office/ODF 文件转 PDF（阶段 C） |
