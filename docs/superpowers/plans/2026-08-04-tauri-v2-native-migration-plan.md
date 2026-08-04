# NutriSurvey Native Tauri v2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace ASP.NET Core and browser launching with a Rust-backed Tauri v2 application using a Next.js static frontend for Linux, Windows, macOS, Android, and iOS.

**Architecture:** Next.js static export runs in Tauri WebView. Tauri commands expose a small serializable DTO boundary to Rust domain modules. Rust owns SQLite, CSV import, nutrition logic, AI HTTP calls, report generation, and native file operations; no local HTTP server remains.

**Tech Stack:** Tauri v2, Rust stable, Next.js static export, React, TypeScript, SQLite, `sqlx`, `serde`, `reqwest`, `csv`, `tauri-plugin-dialog`, `tauri-plugin-fs`, Tauri mobile targets.

## Global Constraints

- Desktop targets are Linux, Windows, and macOS; mobile targets are Android and iOS.
- Frontend uses Next.js static export with no SSR, API routes, server actions, or server-only runtime.
- Rust replaces all ASP.NET Core backend behavior; packaged app requires no .NET, Node, browser, or local HTTP server.
- SQLite lives in platform-native app-data; bundled CSV/template resources are read-only; imported data is copied to app-data.
- AI keys are request-memory-only, never persisted, logged, or placed in URLs.
- Preserve existing CSV formats, food/nutrient behavior, TDEE formulas, meal mapping/scaling, five AI providers, dashboard behavior, and RTF output.
- Commands return serializable DTOs or user-safe structured `AppError` values.
- Tauri capabilities use least privilege; frontend has no arbitrary shell access.
- Do not delete legacy ASP.NET code until Rust feature parity and verification pass.

---

### Task 1: Establish Rust/Tauri/Next workspace

**Files:**
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `next.config.ts`
- Create: `src/app/layout.tsx`
- Create: `src/app/page.tsx`
- Create: `src/lib/commands.ts`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/icons/`
- Modify: `.gitignore`

**Interfaces:**
- Produces `src-tauri/src/lib.rs::run()`, Next static output in `out/`, and a Tauri window loading that output.
- `src/lib/commands.ts` exports `invokeCommand<T>(command: string, payload?: unknown): Promise<T>` as the only frontend-to-Rust boundary.

- [ ] Install exact frontend dependencies: `next`, `react`, `react-dom`, `typescript`, `@tauri-apps/api`, `@tauri-apps/plugin-dialog`, and `@tauri-apps/plugin-fs`.
- [ ] Configure `output: 'export'`, `images.unoptimized: true`, and a static-safe root layout.
- [ ] Initialize Tauri v2 with desktop/mobile configuration, resource paths for `DatabaseMakanan/*.csv` and `Assets/template.rtf`, and a least-privilege default capability.
- [ ] Register dialog and filesystem plugins in Rust and configure only required app-data/read-resource permissions.
- [ ] Add a smoke page that invokes `ping` and displays returned text; add Rust `ping` command.
- [ ] Run `npm run build` and `cargo test --manifest-path src-tauri/Cargo.toml`.
- [ ] Commit: `build: scaffold tauri and next app`.

### Task 2: Define Rust errors, DTOs, paths, and SQLite schema

**Files:**
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/models.rs`
- Create: `src-tauri/src/storage.rs`
- Create: `src-tauri/migrations/0001_initial.sql`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- `AppError` serializes `{ "kind": string, "message": string }` and variants `Validation`, `Database`, `Import`, `Ai`, `Export`, `Io`.
- `Food`, `Nutrient`, `FoodNutrient`, `FoodResult`, `NutrientSummary`, `TdeeRequest`, `TdeeResponse`, `RecommendationFilter`, `AiConfig`, and `MappedMealItem` derive `Serialize`/`Deserialize` with JSON names matching current frontend/backend payloads.
- `Storage::open(app_handle: &AppHandle) -> Result<Storage, AppError>` resolves app-data SQLite and resource directories without current-working-directory assumptions.

- [ ] Add dependencies `serde`, `serde_json`, `thiserror`, `sqlx` with SQLite/runtime-tokio/macros, `tauri`, and `tokio`.
- [ ] Translate `Backend/Models/Food.cs`, `Nutrient.cs`, and `FoodNutrient.cs` into Rust DTOs and database rows.
- [ ] Create schema for foods, nutrients, food_nutrients with foreign keys, indexes for normalized food name and nutrient name, and uniqueness for normalized nutrient names.
- [ ] Add `Storage::initialize()` to create schema and expose a transaction helper.
- [ ] Add tests proving database opens under a supplied app-data path, schema initializes twice safely, and DTO JSON matches existing payload keys.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml storage models`.
- [ ] Commit: `feat: add rust storage and command contracts`.

### Task 3: Port CSV import and food search

**Files:**
- Create: `src-tauri/src/import.rs`
- Create: `src-tauri/src/foods.rs`
- Create: `src-tauri/tests/import_search.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- `import::import_csv(storage: &Storage, bytes: &[u8], source_name: &str) -> Result<u64, AppError>` supports semicolon standard CSV, comma standard CSV, scraper columns `makanan`, `kategori`, `komponen_nutrient_N`, `isi_nutrient_N`, and locale numeric parsing.
- `foods::search(storage: &Storage, query: &str, limit: u32) -> Result<Vec<FoodResult>, AppError>` preserves current empty-query alphabetical-first-five and non-empty max-20 behavior.
- Commands: `food_search`, `food_status`, `nutrient_list`, `import_food_csv`.

- [ ] Port delimiter detection, header normalization, nutrient-unit parsing, scraper-format detection, dynamic nutrient creation, and transactional inserts from `CsvImportService.cs:23-305`.
- [ ] Add resource seeding that imports each bundled CSV exactly once when `foods` is empty, before command readiness is exposed.
- [ ] Implement case-insensitive substring food search and nutrient summaries using the existing nutrient names/aliases.
- [ ] Add integration fixtures for both CSV shapes, comma decimals, malformed rows, duplicate import, empty query, and 20-result cap.
- [ ] Add frontend-safe import command using dialog-selected file bytes, copying the source file into app-data before import.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml --test import_search`.
- [ ] Commit: `feat: port food import and search`.

### Task 4: Port nutrition, TDEE, recommendations, and meal mapping

**Files:**
- Create: `src-tauri/src/nutrition.rs`
- Create: `src-tauri/src/meals.rs`
- Create: `src-tauri/tests/nutrition_meals.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- `nutrition::calculate_tdee(request: TdeeRequest) -> Result<TdeeResponse, AppError>` uses male `66 + 13.7W + 5H - 6.8A`, female/other `655 + 9.6W + 1.8H - 4.7A`, then activity and injury factors.
- `foods::recommend(storage: &Storage, filters: &[RecommendationFilter]) -> Result<Vec<FoodResult>, AppError>` applies all `<`, `>`, `=` filters and caps at 10.
- `meals::map_ai_items(storage: &Storage, items: &[AiMealInput]) -> Result<Vec<MappedMealItem>, AppError>` preserves exact/prefix/contains scoring and nutrient scaling.
- Commands: `calculate_tdee`, `food_recommendations`.

- [ ] Write failing unit tests for both TDEE formulas, factor multiplication, invalid numeric input, nutrient serving scaling, and recommendation operators.
- [ ] Port calculation and alias behavior from `NutritionCalculatorService.cs`, `FoodSearchController.cs`, and frontend aggregation logic.
- [ ] Port AI keyword matching with normalized Indonesian text, candidate limit 25, and current tie-breaking.
- [ ] Add golden tests for mapped item grams/nutrients and filters combining protein, energy, and carbohydrate constraints.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml nutrition_meals`.
- [ ] Commit: `feat: port nutrition and meal domain logic`.

### Task 5: Port AI providers with secure request handling

**Files:**
- Create: `src-tauri/src/ai/mod.rs`
- Create: `src-tauri/src/ai/openai.rs`
- Create: `src-tauri/src/ai/google.rs`
- Create: `src-tauri/src/ai/anthropic.rs`
- Create: `src-tauri/src/ai/prompt.rs`
- Create: `src-tauri/tests/ai.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- `ai::generate_menu(storage: &Storage, request: AiRequest) -> Result<Vec<MappedMealItem>, AppError>`.
- `AiRequest` contains target TDEE/macros, meal types, provider, model, API key, and optional custom base URL; API key is never serialized into logs or persistent state.
- Command: `generate_ai_menu`.

- [ ] Add `reqwest` with TLS and JSON support plus timeouts; define provider-specific request/response structs.
- [ ] Port Indonesian prompt and `meal_plan` schema from `AIService.cs:9-247`; parse raw JSON and fenced JSON safely without exposing response bodies in errors.
- [ ] Implement OpenRouter/OpenAI/custom `/chat/completions`, Gemini `generateContent`, and Anthropic `/messages` adapters with provider URL validation.
- [ ] Ensure Google key is sent only through the supported header/query mechanism without logging it; redact all request configuration in errors.
- [ ] Port local mapping and 5%-tolerance/25-gram scaling behavior through `meals::map_ai_items`.
- [ ] Mock HTTP responses in tests for each provider, malformed JSON, missing fields, HTTP failures, and key-redaction assertions.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml ai`.
- [ ] Commit: `feat: add secure rust ai providers`.

### Task 6: Port RTF export and native file flows

**Files:**
- Create: `src-tauri/src/export.rs`
- Create: `src-tauri/tests/export.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/capabilities/default.json`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- `export::render_rtf(request: ExportRequest, template: &[u8]) -> Result<Vec<u8>, AppError>` returns UTF-8-safe RTF bytes with `Laporan_Nutrisi_yyyyMMdd.rtf` default name.
- Command: `export_word` accepts report DTO and returns bytes or a native save/share result; it never writes an arbitrary path without explicit user selection.

- [ ] Port placeholders, meal rows, nutrient totals, and targets from `ExportController.cs:12-276` into a resource-template renderer.
- [ ] Escape RTF control characters and Unicode food names; preserve Indonesian labels and date-based filename.
- [ ] Add desktop save-dialog flow through Tauri dialog/fs plugins and mobile share/document-picker flow behind a capability-safe abstraction.
- [ ] Test template loading, Unicode, path errors, empty meals, and exact content type/filename semantics.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml export`.
- [ ] Commit: `feat: port native report export`.

### Task 7: Migrate UI to Next.js and invoke adapters

**Files:**
- Create: `src/app/page.tsx`
- Create: `src/components/Navigation.tsx`
- Create: `src/components/Dashboard.tsx`
- Create: `src/components/FoodSearch.tsx`
- Create: `src/components/Recommendations.tsx`
- Create: `src/components/TdeeCalculator.tsx`
- Create: `src/components/AiMealPlanner.tsx`
- Create: `src/components/ReportActions.tsx`
- Create: `src/lib/types.ts`
- Modify: `src/lib/commands.ts`
- Modify: `src/app/layout.tsx`
- Add: `src/styles/globals.css`, local font/icon/sortable assets
- Retire after parity: `Frontend/index.html`, `Frontend/js/app.js`, `Frontend/js/api.js`, `Frontend/css/style.css`

**Interfaces:**
- `src/lib/commands.ts` exports typed functions `searchFoodsByName`, `getNutrientList`, `getRecommendations`, `calculateTdee`, `generateAiMenu`, `exportToWord`, and `importCsv` mapped to command names from Tasks 3–6.
- React state preserves current dashboard shape: meal categories, foods, targets, TDEE result, AI preview, and `.nutri` project serialization.

- [ ] Create failing component tests for search results, TDEE validation/result, recommendation add-to-dashboard, AI preview-to-dashboard, and report action invocation.
- [ ] Convert `Frontend/index.html` sections and `app.js:1-679` state/render flows into client components without changing payload fields or numeric calculations.
- [ ] Replace `fetch`, Blob downloads, FileReader, and CDN dependencies with typed invoke calls and native dialog/fs helpers; bundle SortableJS and visual assets locally.
- [ ] Add versioned `.nutri` serialization (`version: 1`) and import validation, preserving existing project data fields.
- [ ] Add responsive mobile layout while preserving desktop navigation and meal drag/drop behavior.
- [ ] Run `npm test` and `npm run build`; verify generated `out/` contains no server runtime or API route.
- [ ] Commit: `feat: migrate frontend to next invoke ui`.

### Task 8: Configure packaging, mobile targets, and remove launcher dependency

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/capabilities/default.json`
- Modify: `.github/workflows/build.yml` or create it
- Modify: `ReadMe.md`
- Retire after verification: `Assets/RUN.bat`, `Assets/NutriSurvey.vbs`, `Assets/loading.hta`
- Retire after parity: `Backend/`

**Interfaces:**
- `npm run tauri build` produces desktop bundles for configured targets.
- `npm run tauri android build` and `npm run tauri ios build` use platform-specific signing inputs without embedding secrets in repository.

- [ ] Add build scripts for `dev`, `build`, `test`, `tauri:dev`, and `tauri:build`.
- [ ] Configure app identifier, icons, resources, updater metadata, mobile permissions, and platform bundle settings.
- [ ] Add CI matrix for Linux, Windows, macOS and mobile compile/check jobs where runners/toolchains exist; upload unsigned artifacts only when signing secrets are unavailable.
- [ ] Verify no command or frontend code references localhost ports, ASP.NET, `dotnet`, VBS, HTA, or `dotnet serve`.
- [ ] Update README setup/run, data location, import/export, AI key handling, and desktop/mobile build instructions.
- [ ] Remove legacy launcher/backend only after all parity tests and smoke checks pass.
- [ ] Run `npm run lint`, `npm run typecheck`, `npm test`, `cargo fmt --all -- --check`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, and desktop build.
- [ ] Commit: `release: package native tauri application`.

### Task 9: End-to-end verification and migration acceptance

**Files:**
- Create: `tests/e2e/smoke.spec.ts`
- Create: `src-tauri/tests/acceptance.rs`
- Modify: CI workflow and README if required by failures

**Interfaces:**
- Smoke test starts the built static app/Tauri dev host, searches food, calculates TDEE, adds a meal item, generates a mocked AI plan, and exports a report.

- [ ] Seed a test SQLite database with representative rows from both CSV formats.
- [ ] Add acceptance checks for startup readiness, search, recommendations, TDEE, AI provider mocks, import, `.nutri` round-trip, and RTF Unicode output.
- [ ] Run desktop smoke tests on Linux and compile checks for Windows/macOS/mobile targets.
- [ ] Confirm API keys do not occur in logs, generated files, SQLite, URLs, or frontend bundles.
- [ ] Confirm clean packaged start with no .NET/Node/browser/server process dependency.
- [ ] Run the complete verification command set from Task 8 and record platform limitations explicitly in README.
- [ ] Commit: `test: verify native migration acceptance`.
