# Task 9 report

## Evidence

- Frontend: `npm run lint`, `npm run typecheck`, `npm test` (14 tests), and `npm run build` pass.
- Rust: `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`, clippy, and `cargo test --manifest-path src-tauri/Cargo.toml` pass. Full suite: 68 tests; acceptance: 5 tests.
- Acceptance uses repository root from `CARGO_MANIFEST_DIR`, asserts scanned artifact/resource directories exist, scans SQLite/report/static bundle/resources, and exercises configured resource discovery with real temporary standard/scraper CSV files.
- Rust acceptance verifies `src-tauri/src/project.rs` save/load with a real temporary `.nutri` file. Production UI uses `src/lib/project.ts` invoke adapters backed by Rust `project_save`/`project_open` commands; Rust owns native dialog-selected path handling and versioned validation. Frontend contract tests cover command delegation without broad filesystem scope.
- Secret tests use separate synthetic fixtures: Rust acceptance uses a static `acceptance-secret` fixture, sends it through a local mock HTTP server, and checks request authentication plus safe request/debug output; browser smoke uses an invocation-scoped generated fixture, retains only redacted invoke metadata, and scans project/import bytes and loaded `_next` resources. Neither captures a runtime secret or real provider request; browser Tauri invoke is mocked at its boundary.
- CI Rust job builds static frontend before Rust artifact scans and uses correct Cargo manifest path for fmt.

## Unresolved gates

- `npm run test:e2e`: blocked locally because Playwright Chromium executable is unavailable; browser download previously timed out. CI installs Chromium with `npx playwright install --with-deps chromium`. Playwright web server removes `out/`, runs `npm run build`, asserts non-empty `out/index.html`, and serves fresh output with server reuse disabled, so acceptance has no hidden cwd/stale-artifact prerequisite. Smoke assertions cover mocked frontend invoke boundaries, not packaged Tauri runtime logs or live provider traffic.
- `npm run tauri:build`: frontend prerequisite, release binary, `.deb`, and `.rpm` complete; AppImage bundling fails because local `linuxdeploy` is unavailable/incomplete. Packaged-start gate is not passed.
- Mobile report/project delivery is explicitly unsupported in current build because no compatible share/document-picker dependency exists; export never returns `Share` success on mobile. Android packaging was not run locally. Rust Android targets/`adb` availability are not Android Tauri build evidence; CI Android job remains required.
- iOS packaging was not run on Linux because Xcode/macOS toolchain is unavailable; CI macOS job remains required.
- Windows/macOS desktop packaging and clean packaged-start checks remain CI-only.
- No legacy migration files are retired until all listed gates pass with evidence.

## Gate decision

Preserve `Backend/`, `Frontend/`, `Assets/RUN.bat`, `Assets/NutriSurvey.vbs`, and `Assets/loading.hta`.

## Latest verification

- Passed: `npm run lint`, `npm run typecheck`, `npm test` (14/14), `npm run build`, `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, and `cargo test --manifest-path src-tauri/Cargo.toml` (65/65). Custom AI hostnames are resolved before production requests, all resolved addresses are pinned through reqwest `.resolve()` while preserving hostname Host/SNI, redirects are disabled, and every resolved loopback/private/link-local/metadata/reserved address rejects. CSV imports use temporary copies and rename only after successful import; existing same-basename files survive failures. Empty-query parity is documented: backend keeps alphabetical first-five API behavior; UI intentionally requires two characters.
- Blocked: `npm run test:e2e` because local Playwright Chromium executable is unavailable; `npm run tauri:build` because local `linuxdeploy` is unavailable.
- Unresolved: packaged-start, Windows/macOS desktop packaging, Android packaging, iOS packaging, and native runtime-log capture. These remain acceptance blockers, not passed checks.
