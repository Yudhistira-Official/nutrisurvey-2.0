# Task 9 report

## Evidence

- Frontend: `npm run lint`, `npm run typecheck`, `npm test` (13 tests), and `npm run build` pass.
- Rust: `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`, clippy, and `cargo test --manifest-path src-tauri/Cargo.toml` pass. Full suite: 62 tests; acceptance: 5 tests.
- Acceptance uses repository root from `CARGO_MANIFEST_DIR`, asserts scanned artifact/resource directories exist, scans SQLite/report/static bundle/resources, and exercises configured resource discovery with real temporary standard/scraper CSV files.
- Native project acceptance uses production `src-tauri/src/project.rs` save/load abstraction with a real temporary `.nutri` file. Frontend contract coverage verifies `src/app/page.tsx` calls production `serializeProject`/`parseProject` with Tauri filesystem `writeFile`/`readFile`.
- Smoke test records raw invoke data only transiently to prove mocked API key is seen in-memory, retains only payload keys/redacted diagnostics, scans serialized calls/log state/project/import bytes, and scans loaded `_next` bundle resources. Raw secret is asserted absent from all retained/output surfaces.
- CI Rust job builds static frontend before Rust artifact scans and uses correct Cargo manifest path for fmt.

## Unresolved gates

- `npm run test:e2e`: blocked locally because Playwright Chromium executable is unavailable; browser download previously timed out. CI installs Chromium with `npx playwright install --with-deps chromium`.
- `npm run tauri:build`: release binary plus `.deb`/`.rpm` packaging reach AppImage, then fail because local `linuxdeploy` is unavailable/incomplete. Packaged-start gate is not passed.
- Android packaging was not run locally. Rust Android targets/`adb` availability are not Android Tauri build evidence; CI Android job remains required.
- iOS packaging was not run on Linux because Xcode/macOS toolchain is unavailable; CI macOS job remains required.
- Windows/macOS desktop packaging and clean packaged-start checks remain CI-only.
- No legacy migration files are retired until all listed gates pass with evidence.

## Gate decision

Preserve `Backend/`, `Frontend/`, `Assets/RUN.bat`, `Assets/NutriSurvey.vbs`, and `Assets/loading.hta`.
