# NutriSurvey 2.0

NutriSurvey is a native Tauri v2 nutrition analysis application. It uses a Next.js static frontend and Rust commands for food search, nutrition calculations, AI meal planning, SQLite storage, CSV import, and RTF report export.

## Features

- Local food and nutrient database with CSV import.
- Serving-aware nutrient totals and TDEE calculation.
- Nutrient recommendations and AI meal plans mapped to local foods.
- RTF report export through native save/share flows.
- Responsive desktop and mobile UI.

## Development setup

Requirements: Node.js 22+, npm, Rust stable, and the platform dependencies documented by Tauri for the target platform.

```sh
npm ci
npm run dev
```

Run the native development shell with:

```sh
npm run tauri:dev
```

The frontend is exported as static files into `out/`; packaged applications do not start an HTTP API, browser launcher, ASP.NET runtime, or other local server.

## Build and checks

```sh
npm run lint
npm run typecheck
npm test
npm run build
cargo fmt --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri:build
```

Desktop bundles are written below `src-tauri/target/release/bundle`. `tauri build` runs `npm run build` automatically through `beforeBuildCommand`, so stale or missing `out/` content is not packaged. Signing credentials are supplied by the CI environment or local keychain and are never committed.

Updater support is intentionally disabled for this migration release: no public key, endpoint, or signing metadata is present. Enable Tauri updater only in a release change after generating a real keypair, publishing signed update artifacts, and storing the public key plus endpoint as reviewed repository configuration; keep private signing keys in CI secrets.

Mobile project setup and builds:

```sh
npx tauri android init
npm run tauri:build -- android
npx tauri ios init
npm run tauri:build -- ios
```

Android CI installs Android platform 35, build-tools 35.0.0, NDK 27.2.12479018, and all Rust Android targets before initializing and building the Tauri project. Android signing uses Gradle/Android keystore environment or runner secrets. iOS signing uses Xcode, certificates, and provisioning profiles supplied through the runner. Without signing inputs, CI performs compile/build checks and uploads unsigned outputs where available.

## Data and files

- SQLite database is stored in the platform-native application data directory.
- Bundled CSV files under `DatabaseMakanan/` and `Assets/template.rtf` are read-only packaged resources.
- Imported CSV files are copied into the application data `imports` directory before import.
- Desktop reports use a user-selected save path. Mobile export uses the native delivery abstraction.
- Existing `.nutri` project data remains application-managed and is not written to repository paths.

## AI key handling

AI uses BYOK. Provider, model, optional custom base URL, and API key are sent only for the active request. Keys are request-memory-only: they are not persisted in SQLite, URLs, logs, frontend bundles, packaged resources, or generated reports. Do not commit keys or place them in CI configuration files.

Supported providers: OpenRouter, OpenAI-compatible endpoints, Google Gemini, Anthropic, and custom OpenAI-compatible routers.

## Migration status

Rust/Tauri feature parity and frontend migration are implemented through Task 7. Legacy `Backend/`, `Frontend/`, and launcher files remain intentionally during the Task 9 acceptance window. They must not be used for native development or packaging and will be retired only after parity tests, smoke checks, and clean packaged-start verification pass.

## CI

`.github/workflows/build.yml` runs frontend, Rust, Linux, Windows, macOS, Android, and iOS checks where GitHub-hosted toolchains exist. Signing is conditional on repository/organization secrets. Unsigned artifacts are uploaded when signing secrets are unavailable; secrets are never embedded in the repository. Frontend smoke runs against the static `out/` app with a mocked Tauri invoke bridge.

## Task 9 acceptance status

Native acceptance coverage includes startup/readiness, both CSV fixture shapes, search, recommendations, TDEE, mocked AI mapping and key redaction, `.nutri` version contract, and Unicode RTF rendering. Local verification passed frontend lint/typecheck/tests, Rust fmt/clippy/tests, static build, and native release binary compilation.

Legacy `Backend/`, `Frontend/`, and launcher files remain because migration retirement requires every parity gate to pass. Linux packaging is blocked locally by missing `linuxdeploy`; Playwright smoke could not run because Chromium download timed out. Android Rust targets and `adb` are installed, but Android build was not completed in this environment. iOS build requires macOS/Xcode and is unavailable here. CI remains authoritative for Windows, macOS, Android, iOS, and hosted packaged-start checks.
