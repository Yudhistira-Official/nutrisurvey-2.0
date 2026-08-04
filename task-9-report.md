# Task 9 report

## Acceptance coverage

- Rust acceptance fixtures cover readiness, bundled seeding from representative standard and scraper CSV resources, search, recommendations, TDEE, mocked AI mapping, API-key redaction, `.nutri` file write/read/re-import, Unicode RTF output, and artifact secret scans.
- Frontend smoke covers readiness, food search, recommendations/add-to-dashboard, CSV import result, real page save/load project flow, TDEE, mocked AI implementation, and report export outcome. Tauri dialog/filesystem calls are mocked only at the browser boundary.
- CI runs frontend smoke and uses `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`.

## Verification status

Checks rerun after review fixes: Rust acceptance 3/3, full Rust suite 60 tests, clippy, fmt, frontend tests 12/12, lint, typecheck, and static build passed. Packaged build was attempted and stopped at the local `linuxdeploy` blocker; it is not a passing packaged-build gate.

Playwright smoke remains environment-dependent: local Chromium executable was unavailable and browser download timed out. CI provisions Chromium with `npx playwright install --with-deps chromium`.

Linux packaged bundle remains blocked locally by missing/incomplete `linuxdeploy`. Android build was not run locally; Rust Android targets and `adb` availability do not prove Tauri Android packaging. iOS build was not run because Linux has no Xcode/macOS toolchain. Windows/macOS packaged-start checks remain CI-only.

## Gate decision

Legacy `Backend/`, `Frontend/`, `Assets/RUN.bat`, `Assets/NutriSurvey.vbs`, and `Assets/loading.hta` remain preserved. Retirement gate is closed until smoke, packaged-start, desktop bundle, Android, iOS, and secret-scan gates all pass with evidence.

## Review follow-up

- Acceptance now derives repository root from `CARGO_MANIFEST_DIR`; artifact/resource paths are not dependent on test working directory, and every scanned directory is asserted present before traversal.
- `project::save` and `project::load` provide production Rust `.nutri` serialization/file handling. Acceptance writes a real temp `.nutri`, reloads it through the production loader, and compares the complete typed project.
- `seed_configured_resources` opens an explicit configured resource directory, discovers sorted CSV files, imports real representative standard/scraper fixtures, verifies readiness and idempotence, and cleans up temp app-data/resources.
- Smoke records serialized invoke calls/log state, checks project/import byte payloads for secrets, and fetches every loaded `_next` resource to scan non-empty bundle contents for the mocked secret.
- Latest local evidence: Rust acceptance 5/5, full Rust suite 62 tests, clippy/fmt, frontend tests 12/12, lint/typecheck/build passed. Playwright still lacks Chromium; packaged Linux build still stops at `linuxdeploy`; mobile/other desktop gates remain CI-only.
- Rust CI now installs Node, runs the static frontend build before Rust tests, and then runs `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`; this ensures repo-root `out/` artifact scans have configured inputs.
