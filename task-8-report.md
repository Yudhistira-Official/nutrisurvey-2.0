# Task 8 report

## Implemented

- Added `dev`, `lint`, `tauri:dev`, and `tauri:build` scripts; retained build/test/typecheck scripts.
- Added pinned Tauri CLI, ESLint, and Next ESLint configuration. Lint ignores generated Next/Tauri output.
- Configured Tauri bundle metadata, application icon, CSV/RTF resources, automatic static frontend build via `beforeBuildCommand`, and unsigned-build behavior.
- Changed bundle identifier to `com.nutrisurvey.desktop`, avoiding the macOS `.app` suffix warning.
- Disabled updater configuration until real signed release metadata exists; documented enablement requirements without fake URLs or secrets.
- Expanded capability configuration for native open/save flows, app-data writes, packaged-resource reads, and declared desktop/mobile platforms.
- Added GitHub Actions frontend, Rust, desktop, Android, and iOS compile/build jobs with frontend builds before Tauri packaging, explicit Android SDK/NDK versions, Rust Android targets, conditional unsigned artifact uploads, and secret-only signing inputs.
- Rewrote README for native setup, data locations, import/export, AI key handling, desktop/mobile builds, CI, and migration retirement gates.
- Kept `Backend/`, `Frontend/`, `Assets/RUN.bat`, `Assets/NutriSurvey.vbs`, and `Assets/loading.hta`; Task 9 parity and acceptance checks are not complete.

## Verification

- `npm run lint` passed.
- `npm run typecheck` passed.
- `npm test` passed: 11 tests.
- `npm run build` passed: static `/` and `/_not-found` routes.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed: 57 tests, 0 failures.
- `npx tauri build --ci` ran `beforeBuildCommand` successfully, compiled the release binary, and produced `.deb` and `.rpm`; AppImage bundling failed in local environment because `linuxdeploy` did not complete.
- Tauri environment reports Linux WebKitGTK/rsvg2, stable Rust, Node, npm, and Tauri CLI available.
- Android CLI help is available, but Android SDK/NDK and configured mobile project are unavailable locally; Android build not run. CI now installs platform 35, build-tools 35.0.0, NDK 27.2.12479018, and all Rust Android targets before building.
- iOS CLI is unavailable on Linux; iOS build not run. CI job targets macOS.
- Workflow YAML parser is unavailable locally because Ruby is not installed; GitHub Actions remains required validation.
- Source scan found no localhost, ASP.NET, dotnet, VBS, HTA, or `dotnet serve` references in `src/`; legacy `Frontend/` and migration-plan references remain intentionally.

## Concerns / follow-up

- Task 9 must complete parity, smoke, and clean packaged-start checks before legacy backend and launchers can be deleted.
- Updater remains disabled; enable only with a real generated public key, published signed artifacts, reviewed endpoint, and CI-held private signing key.
- Install Linux AppImage tooling or use CI to produce AppImage artifacts.
- Validate Tauri mobile capability generation and native share/document delivery during Task 9 on Android and macOS/iOS runners.
