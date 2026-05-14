# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
# Package manager: pnpm (required)

# Frontend dev server only (port 1420)
pnpm dev

# Type-check and build frontend → dist/
pnpm build

# Tauri dev (frontend + Rust backend together, port 1420 → Tauri window)
pnpm tauri dev

# Tauri production build
pnpm tauri build

# Rust: build backend only
cd src-tauri && cargo build

# Rust: check compilation without linking
cd src-tauri && cargo check

# Rust: run tests (in-memory SQLite, no Tauri runtime needed)
cd src-tauri && cargo test
```

TypeScript is pinned to `~5.6.2`. The Rust toolchain must be **1.88.0 or newer** (transitive deps `darling`, `plist`, `serde_with`, `time` require it). Run `rustup update stable` if you see rustc version errors.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Tauri 2 desktop shell                                  │
│  Window: 900×620 (min 680×480), title "网络打印服务"    │
│  decorations: false (custom TitleBar.vue titlebar)      │
│  Dev: localhost:1420, CSP: null                         │
├────────────────────┬────────────────────────────────────┤
│  Frontend (Vue 3)  │  Backend (Rust)                    │
│  src/              │  src-tauri/src/                    │
│                    │                                    │
│  5 routes (hash):  │  main.rs → print_lib::run()       │
│   /files  HomePage │  lib.rs: Builder + 22 commands     │
│   /print  Queue    │  commands.rs: 16 Tauri commands    │
│   /history         │  db.rs: SQLite init + migrations   │
│   /log   SystemLog │  entities.rs: PrintJob, SystemLog  │
│   /settings        │  repos.rs: CRUD + 7 unit tests     │
│                    │  http_server.rs: axum LAN server    │
│  7 Pinia stores    │  qr.rs: QR code → base64 PNG       │
│  5 composables     │  network.rs: get_local_ip()         │
│  4 layout comps    │  logger.rs: info/warn/error helpers │
│                    │                                    │
│                    │  Plugins: opener, store, dialog     │
│                    │  SQLite: app.db (rusqlite bundled)  │
│                    │  LAN HTTP: axum on port 5000        │
│                    │                                    │
│                    │  ┌─────────────────────────┐       │
│                    │  │ Java Print Service      │       │
│                    │  │ (external, port 2024)   │       │
│                    │  │ HTTP: getPrintServers,  │       │
│                    │  │       print/single       │       │
│                    │  │ WS:   /print            │       │
│                    │  └─────────────────────────┘       │
└────────────────────┴────────────────────────────────────┘
```

- **Frontend** (`src/`): Vue 3 Composition API (`<script setup>`). Ant Design Vue 4.x for UI. Vue Router (hash mode) with 5 lazy-loaded routes. Pinia stores for state. The `@tauri-apps/api/core` `invoke()` calls Rust commands; `fetch()` calls the external Java Print Service; WebSocket connects to Java for real-time print status.
- **Backend** (`src-tauri/`): Tauri 2 Rust app. `main.rs` is a thin entry point; `lib.rs` sets up plugins, initializes SQLite, generates a LAN auth token, spawns the axum HTTP server on port 5000, and registers all commands.
- **Lib naming**: The Rust library is named `print_lib` (with `_lib` suffix) to avoid a bin/lib name conflict on Windows — this is a Tauri scaffolding quirk, don't rename it.
- **Dual persistence**: App settings (print defaults, allowed extensions, UI state) use `tauri-plugin-store` (JSON files at `app_data_dir`). Print jobs and system logs use SQLite (`app.db` at `app_data_dir`) via `rusqlite` with bundled SQLite.
- **Custom window chrome**: `decorations: false` in tauri.conf.json. `TitleBar.vue` handles minimize/maximize/close via `@tauri-apps/api/window`. The maximize button tracks window state via `onResized` listener.
- **Dual layout**: `App.vue` detects viewport width via `usePlatform()` (breakpoint 768px). Desktop gets sidebar nav + custom titlebar; mobile gets bottom tab bar. Sidebar auto-collapses at 1024px via `useNavCollapse()`.
- **Java Print Service dependency**: This app is a client to an external Java Spring Boot service (default `http://localhost:2024`). The Java service owns printer enumeration and print execution. The Tauri app manages files locally and sends them to Java for printing.
- **LAN mobile upload**: At startup, the Rust backend spawns an axum HTTP server on port 5000. It serves an embedded mobile upload page at `/` and accepts multipart file uploads at `POST /upload` with token auth, extension whitelist, and 50MB size limit. A QR code encoding the LAN URL (with auth token) is shown in the UI so mobile devices can scan and upload files.
- **File transfer via base64**: The `file_read` Tauri command returns file contents as base64-encoded strings. The frontend converts these to Blob URLs or ArrayBuffers for preview.

## Key Frontend Patterns

### Stores (Pinia)
| Store (`src/stores/`) | Persistence | Key State |
|------------------------|-------------|-----------|
| `app-config` | Tauri Store | serviceHost/Port, lanPort, computed serviceUrl/wsUrl |
| `settings` | Tauri Store | allowedExtensions, print defaults, lanPort, autoStart |
| `printer-list` | Memory only | printers[], defaultPrinter |
| `print-task` | Memory only | currentStatus, currentJobName, isActive |
| `print-history` | Via Tauri SQLite commands | records[], filters |
| `system-log` | Via Tauri SQLite commands | logs[], filters |
| `file-browser` | Memory | files[], sortBy |

### Composables (`src/composables/`)
- `usePlatform()` — mobile/desktop detection at 768px breakpoint
- `useFileSystem()` — wraps Tauri `file_save/read/delete/list` commands with base64 helpers
- `usePrintService()` — orchestrates full print flow: WebSocket connect → create job record → POST to Java service
- `useWebSocket()` — WebSocket with exponential backoff reconnection (0, 1s, 2s, 4s, 8s, then 30s)
- `useNavCollapse()` — sidebar collapse state, auto at 1024px, manual toggle persisted to Tauri Store

### API Layer (`src/api/`)
- `http-client.ts` — configurable `fetch()` wrapper targeting the Java service
- `print-api.ts` — `getPrintServers()` and `printSingle()` calls
- `websocket-client.ts` — typed WebSocket factory for print status streaming
- `types.ts` — `PrintStatusCode` enum, `PrintStatus` union, `PaperSizes` constants

### File Preview (`src/views/file-preview/`)
- `PdfPreview.vue` — uses `pdfjs-dist` for canvas-based PDF rendering with page navigation and zoom
- `ImagePreview.vue` — Blob URL rendering with zoom controls (25%-500%)
- `TextPreview.vue` — monospace `<pre>` display
- `HtmlPreview.vue` — sandboxed `<iframe srcdoc>` rendering

## Key Backend Patterns

### SQLite Schema (migrations in `db.rs`)
- `print_jobs` — id, name, status, printer, print_type, source, copies, file_path, file_size, error_msg, created_at, finished_at
- `system_logs` — id, timestamp, level, category, message, logger
- `app_config` — key, value, updated_at

### Tauri Commands (all in `commands.rs`)
Print jobs: `print_jobs_list`, `print_jobs_create`, `print_jobs_get_by_id`, `print_jobs_delete`, `print_jobs_update_status`, `print_jobs_count_queue`, `print_jobs_count_today`
Logs: `log_insert`, `log_query`, `log_clear`
Files: `file_save`, `file_read`, `file_delete`, `file_list`
LAN: `lan_server_url`, `lan_server_qrcode`

### Managed State
- `AppState` — `Mutex<Connection>` (SQLite)
- `LanServerToken` — `String` (32-char hex auth token for LAN uploads)

### Capabilities (`src-tauri/capabilities/default.json`)
Window manipulation permissions: minimize, maximize, toggle-maximize, close, is-maximized, start-dragging, set-focus, is-focused. Core defaults for opener, store, and events.

## Project Conventions

- **Ant Design Vue 严格规范**: 所有 UI 必须使用 antd 组件，严禁使用原生 HTML 表单/媒体/排版元素。写模板前通过 antd MCP 查对应组件（`mcp__antd__antd_list` 列出全部，`mcp__antd__antd_doc` 查用法）。`<div>` 和结构性 `<span>` 除外。
- TypeScript `strict: true`, `noUnusedLocals: true`, `noUnusedParameters: true`.
- No linter or formatter is configured (no ESLint, Prettier, or rustfmt overrides).
- `.gitignore` excludes `.claude/` — local Claude settings won't be committed.
- Vue components use `<script setup lang="ts">` with Composition API.
- Rust repos (`repos.rs`) contain both CRUD implementations and unit tests (in-memory SQLite, `#[cfg(test)]` module).
- Frontend styling uses 6 global CSS files in `src/assets/styles/` imported in `main.ts` — not scoped/component styles.
