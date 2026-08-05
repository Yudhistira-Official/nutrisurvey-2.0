# Settings History and AI Configuration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a gear settings panel with validated project/report history, native file opening, and persisted AI configuration.

**Architecture:** Rust stores history paths in SQLite `app_state`, validates existence, and opens files through a Tauri shell capability. React renders settings modal and reuses existing AI keychain/local preference behavior.

**Tech Stack:** Tauri v2, Rust, SQLite/SQLx, Tauri shell plugin, Next.js/React, TypeScript.

## Global Constraints

- Never store file contents in history; store paths only.
- Hide missing paths from lists.
- Never expose API key in history, localStorage, logs, or project files.
- Native open accepts only existing regular files.
- Preserve existing save/report behavior.
- Do not commit secrets or `.env`.

---

### Task 1: Backend history and native open commands

**Files:** `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/src/storage.rs`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, Rust tests.

- Add shell plugin and `shell:allow-open` capability.
- Add commands to record project/report paths, list existing paths, and open an existing regular file with native OS handler.
- Persist JSON history in `app_state`; filter missing paths before returning.
- Call record commands after successful project save and report save.
- Add tests for missing-path filtering and invalid open paths.

### Task 2: Frontend command adapters and Settings modal

**Files:** `src/lib/commands.ts`, `src/lib/types.ts`, `src/components/Navigation.tsx`, new `src/components/SettingsPanel.tsx`, `src/app/page.tsx`.

- Add typed history records and command adapters.
- Add gear button below sidebar actions.
- Settings modal loads history on open, shows Project History and Word Reports, and invokes native open only for returned paths.
- Add AI Provider, Model, Base URL, API key, remember-key checkbox, and save configuration action. Reuse keychain commands; never display key contents.
- Refresh history after saves/reports.

### Task 3: UI and verification

**Files:** `src/styles/globals.css`, frontend/Rust tests.

- Style gear button, modal sections, history rows, empty states, and responsive layout with consistent spacing.
- Run Rust tests serially where existing current-directory tests require it, frontend tests, typecheck, lint, and diff check.
- Confirm `.env` and `.opencode` remain ignored.
