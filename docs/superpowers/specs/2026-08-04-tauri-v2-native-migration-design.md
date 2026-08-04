# NutriSurvey Native Tauri v2 Design

## Goal

Migrate NutriSurvey 2.0 from ASP.NET Core + browser launcher to one native Tauri v2 application for Linux, Windows, macOS, Android, and iOS. Preserve existing nutrition features while replacing the backend with Rust.

## Architecture

Next.js static export renders the existing UI inside Tauri WebView. Frontend calls thin Tauri commands through `invoke()`; no local HTTP server remains. Rust owns domain logic, persistence, integrations, and native file operations.

Rust modules:

- nutrition: food nutrient calculation and TDEE
- meals: meal mapping, scaling, dashboard implementation
- import: CSV parsing and database seeding/import
- ai: provider adapters and response validation
- export: Word report generation
- storage: platform app-data paths and SQLite access
- commands: validated Tauri command boundary

SQLite is stored in the platform-native app-data directory. Seed CSV files are bundled as application resources. Writable imported databases remain in app data.

## Frontend

Convert the current Vanilla JS/HTML/CSS SPA to Next.js static export. Next.js must not use SSR, API routes, or server-only runtime behavior. Existing visual behavior and feature flows remain equivalent unless platform capability requires a native dialog or share/save flow. Frontend API calls become typed invoke adapters.

## Commands and data flow

Commands accept explicit DTOs, validate inputs, call domain services, and return serializable DTOs or structured errors. Planned command groups cover food search, nutrient lists, recommendations, TDEE, AI menu generation, CSV import, and Word export.

Rust uses an `AppError` enum with user-safe serialized categories for validation, database, import, AI, export, and filesystem failures. Internal secrets and stack details are never returned to the UI.

## AI and security

AI provider requests originate in Rust. BYOK keys are held only in request memory, never persisted, logged, or put in URLs. Provider adapters support OpenRouter, OpenAI, Google Gemini, Anthropic, and custom OpenAI-compatible routers. Responses are schema-validated and mapped against the local SQLite food database before display.

Tauri capabilities use least privilege. File selection and writes require explicit user interaction and scoped paths. No arbitrary shell execution is exposed to the frontend.

## Native capabilities

Use Tauri v2 plugins where needed for filesystem, dialog, notification, updater, and shell capabilities. Export uses a save dialog. Mobile behavior must degrade cleanly where desktop-only file semantics differ; sharing or document picker integration is used where supported.

## Data compatibility

Preserve existing CSV formats, food fields, nutrient fields, AI meal behavior, five AI providers, TDEE calculations, dashboard menus, and Word report output. Existing databases are migrated or recreated through a documented import path; no ASP.NET runtime is required after migration.

## Testing

Rust unit tests cover TDEE, nutrient calculations, meal mapping, AI scaling, and CSV parsing. SQLite integration tests cover schema, seed/import, search, and recommendations. Command tests cover DTO serialization and error mapping. Frontend tests cover invoke adapters, state transitions, and primary forms. CI builds and tests desktop targets plus mobile targets where toolchains are available.

## Delivery stages

1. Scaffold Tauri v2 and Next.js static export without removing existing behavior.
2. Define SQLite schema, Rust models, errors, and storage paths.
3. Port nutrition, food search, recommendations, TDEE, CSV import, and meal logic.
4. Port AI providers and secure request handling.
5. Port Word export and native dialogs.
6. Replace frontend API client with invoke adapters and migrate UI pages.
7. Add capabilities, packaging, updater configuration, and platform builds.
8. Run tests and validate desktop/mobile builds.

## Success criteria

The packaged application starts without .NET, Node, a browser, or a local HTTP server; existing core features work through Rust commands; user data resides in platform app data; AI keys are not persisted; and builds are available for Linux, Windows, macOS, Android, and iOS subject to platform signing requirements.
