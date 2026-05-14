# AGENTS.md

## Build Commands

- **Frontend dev** (port 1420): `pnpm dev`
- **Frontend build**: `pnpm build` — runs `vue-tsc --noEmit && vite build`
- **Tauri dev** (frontend + Rust): `pnpm tauri dev`
- **Tauri production build**: `pnpm tauri build`
- **Rust only**: `cd src-tauri && cargo build` or `cargo check`

## Important Constraints

- **Package manager**: pnpm (required), not npm/yarn
- **TypeScript**: pinned to `~5.6.2`
- **Rust toolchain**: 1.88.0 or newer required (deps `darling`, `plist`, `serde_with`, `time` need it). Run `rustup update stable` if you see rustc version errors.
- **No linter/formatter** configured — ESLint, Prettier, rustfmt are all absent.
- **No tests** exist yet.
- **TypeScript strict mode** enabled: `strict: true`, `noUnusedLocals: true`, `noUnusedParameters: true`.

## Architecture

- **Rust lib name**: `print_lib` (with `_lib` suffix) — this is a Tauri Windows workaround to avoid a bin/lib name conflict. Do not rename.
- **Lib entry**: `src-tauri/src/main.rs` calls `print_lib::run()`, which lives in `lib.rs`.
- **IPC**: Frontend calls Rust via `@tauri-apps/api/core`'s `invoke()`. Commands are registered with `tauri::generate_handler![]` in `lib.rs`.
- **Window**: 800×600, frameless (`decorations: false`), title "网络打印服务", CSP is null (unrestricted).
- **Dev server**: `localhost:1420`, strict port — fails if occupied.

## Key Files

- `src-tauri/src/lib.rs` — Tauri builder, plugins, setup (DB init, LAN server spawn)
- `src-tauri/src/commands.rs` — all Tauri commands (`print_jobs_*`, `file_*`, `log_*`, `lan_server_*`)
- `src-tauri/src/http_server.rs` — LAN HTTP upload server (port 5000, token auth)
- `src-tauri/src/db.rs` — SQLite init via `rusqlite` (bundled)
- `src/main.ts` — Vue app entry
- `src/App.vue` — root component
- `src/router/index.ts` — Vue Router setup