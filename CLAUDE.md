# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
# Package manager: pnpm (required)

# Frontend dev server (port 1420)
pnpm dev

# Type-check and build frontend → dist/
pnpm build

# Tauri dev (starts frontend + Rust backend together)
pnpm tauri dev

# Tauri production build
pnpm tauri build

# Rust: build backend only
cd src-tauri && cargo build

# Rust: check compilation without linking
cd src-tauri && cargo check
```

TypeScript is pinned to `~5.6.2`. The Rust toolchain must be **1.88.0 or newer** (transitive deps `darling`, `plist`, `serde_with`, `time` require it). Run `rustup update stable` if you see rustc version errors.

## Architecture

```
┌─────────────────────────────────────────────┐
│  Tauri 2 desktop app shell                  │
│  Window: 800×600, title "print"             │
│  Dev server: localhost:1420                 │
│  CSP: null (unrestricted)                   │
├──────────────┬──────────────────────────────┤
│  Frontend    │  Backend (Rust)              │
│  Vue 3 + TS  │  src-tauri/src/              │
│  src/        │                              │
├──────────────┤  main.rs → calls             │
│  main.ts     │    print_lib::run()          │
│  App.vue     │                              │
│              │  lib.rs defines:             │
│              │  - greet() Tauri command     │
│              │  - run() sets up Builder     │
│              │    with opener plugin         │
└──────────────┴──────────────────────────────┘
```

- **Frontend** (`src/`): Vue 3 with `<script setup>` Composition API. Single `App.vue` component with a `greet` form that calls the Rust backend via `invoke("greet", { name })`. Vite is configured with `clearScreen: false` (so Rust errors aren't hidden), strict port 1420, and `src-tauri/` is excluded from file watching.
- **Backend** (`src-tauri/`): Tauri 2 Rust app. `main.rs` is a thin entry point; `lib.rs` holds the Tauri builder setup and commands. The single command `greet(name: &str) -> String` is invoked from the frontend. The `tauri-plugin-opener` is the only plugin enabled.
- **Lib naming**: The Rust library is named `print_lib` (with `_lib` suffix) to avoid a bin/lib name conflict on Windows — this is a Tauri scaffolding quirk, don't rename it.
- **Inter-process communication**: Frontend calls Rust via `@tauri-apps/api/core`'s `invoke()`. Commands are registered with `tauri::generate_handler![]` in `lib.rs`.

## Project Conventions

- No linter or formatter is configured yet (no ESLint, Prettier, or rustfmt overrides).
- No tests exist yet.
- TypeScript `strict: true`, `noUnusedLocals: true`, `noUnusedParameters: true`.
- `.gitignore` already excludes `.claude/` — local Claude settings won't be committed.
