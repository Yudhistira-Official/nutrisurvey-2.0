# Food Database Auto-Update Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Synchronize bundled and external food CSV files at application startup only when their SHA-256 manifest changes.

**Architecture:** Keep startup discovery in `src-tauri/src/lib.rs`, add manifest-aware transactional synchronization in `src-tauri/src/import.rs`, and use SQLite `app_state` for the persisted manifest. When sources change, clear food/nutrient rows, import all sources in sorted order, and write the manifest inside one transaction; unchanged sources skip import.

**Tech Stack:** Rust 2021, Tauri v2, Tokio, SQLx SQLite, `sha2`, existing CSV importer, Rust integration tests.

## Global Constraints

- Source set includes bundled resource CSV files and external `DatabaseMakanan` CSV files.
- Manifest uses SHA-256 content hashes and deterministic source ordering.
- Import, database replacement, and manifest update must be atomic.
- Failed synchronization must preserve previous data and manifest.
- No unrelated frontend or migration refactor.
- Do not commit existing unrelated workspace changes.

---

### Task 1: Add SHA-256 dependency and manifest storage helpers

**Files:**
- Modify: `src-tauri/Cargo.toml:16-27`
- Modify: `src-tauri/src/import.rs:1-58,366-379`
- Test: `src-tauri/tests/import_search.rs`

**Interfaces:**
- Produces `pub struct SourceManifest` or equivalent serializable manifest representation.
- Produces `pub async fn synchronize_sources(storage: &Storage, resources: &[(&str, &[u8])]) -> Result<u64, AppError>`.
- Existing `seed_csvs` behavior remains available for existing tests.

- [ ] **Step 1: Add failing manifest tests**

Add tests to `src-tauri/tests/import_search.rs` covering:

```rust
#[tokio::test]
async fn source_manifest_changes_only_when_source_content_changes() {
    let (storage, path) = storage().await;
    let first = [("foods.csv", b"Nama Makanan\nRice\n".as_slice())];
    assert_eq!(import::synchronize_sources(&storage, &first).await.unwrap(), 1);
    assert_eq!(foods::search(&storage, "rice", 20).await.unwrap().len(), 1);
    assert_eq!(import::synchronize_sources(&storage, &first).await.unwrap(), 0);
    cleanup(path).await;
}
```

Also add a changed-content case asserting the old food is replaced by the new food, and an invalid CSV case asserting the prior food remains searchable after the error.

- [ ] **Step 2: Run focused tests and verify failure**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test import_search source_manifest -- --nocapture
```

Expected: FAIL because synchronization API does not exist.

- [ ] **Step 3: Add dependency**

Add:

```toml
sha2 = "0.10"
```

- [ ] **Step 4: Implement deterministic SHA-256 manifest**

In `import.rs`, hash each `(source_name, bytes)` pair with SHA-256, sort resources by source name before hashing/import, and serialize manifest as stable JSON. Store it under `app_state` key `food_sources_manifest`.

Expose helpers that read the existing manifest and compare it to the current manifest. A missing manifest must count as changed when resources exist.

- [ ] **Step 5: Implement atomic synchronization**

Open one transaction. If current manifest equals stored manifest, rollback/return `0` without modifying food data. If changed:

```sql
DELETE FROM food_nutrients;
DELETE FROM foods;
```

Import all sources using existing transaction-level importer, set `seed_complete = true`, then upsert `food_sources_manifest`. Commit only after all operations succeed. Any error must drop the transaction and preserve previous data/manifest.

- [ ] **Step 6: Run focused tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test import_search source_manifest -- --nocapture
```

Expected: PASS.

### Task 2: Use synchronization during Tauri startup

**Files:**
- Modify: `src-tauri/src/lib.rs:229-296`
- Test: `src-tauri/tests/acceptance.rs:142-174`

**Interfaces:**
- `seed_configured_resources` continues discovering resources, but calls `import::synchronize_sources`.
- Existing resource directory search order remains compatible with development and packaged builds.

- [ ] **Step 1: Add startup skip/change tests**

Extend `native_acceptance_seeds_configured_resource_dir_from_real_csv_files` to call `seed_configured_resources` twice and assert the second result is `0`. Modify one resource file, call startup seeding again, assert the result is nonzero and the changed food is searchable while the removed food is not.

- [ ] **Step 2: Run acceptance test and verify failure**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test acceptance native_acceptance_seeds_configured_resource_dir_from_real_csv_files -- --nocapture
```

Expected: FAIL until startup uses manifest-aware synchronization.

- [ ] **Step 3: Replace startup seed call**

Change `seed_configured_resources` line 295 from `import::seed_csvs(storage, &references).await` to `import::synchronize_sources(storage, &references).await`.

Keep sorted resource discovery. Deduplicate paths if bundled and external lookup point to the same files, preventing duplicate source entries and unstable manifests.

- [ ] **Step 4: Handle no-source startup explicitly**

If no CSV files are discovered, return `0` without clearing existing food data or writing a new manifest. This protects development/test environments where resource paths are unavailable.

- [ ] **Step 5: Run acceptance tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test acceptance native_acceptance_seeds_configured_resource_dir_from_real_csv_files -- --nocapture
```

Expected: PASS.

### Task 3: Full verification and regression review

**Files:**
- Modify only files listed above unless tests expose a required compile fix.

- [ ] **Step 1: Run all Rust tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 2: Run frontend checks**

Run:

```bash
npx tsc --noEmit
npm run lint
```

Expected: PASS; frontend changes are unrelated and must not be modified.

- [ ] **Step 3: Review diff and workspace safety**

Run:

```bash
git diff -- src-tauri/Cargo.toml src-tauri/src/import.rs src-tauri/src/lib.rs src-tauri/tests/import_search.rs src-tauri/tests/acceptance.rs docs/superpowers/specs/2026-08-04-food-database-auto-update-design.md docs/superpowers/plans/2026-08-04-food-database-auto-update-plan.md
git status --short
```

Confirm unrelated pre-existing modifications are untouched and no secrets are added.

- [ ] **Step 4: Commit intended implementation files**

Only after user explicitly requests commit. Stage only auto-update files and use a concise conventional commit message such as:

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/import.rs src-tauri/src/lib.rs src-tauri/tests/import_search.rs src-tauri/tests/acceptance.rs docs/superpowers/specs/2026-08-04-food-database-auto-update-design.md docs/superpowers/plans/2026-08-04-food-database-auto-update-plan.md
git commit -m "feat: auto-update food database sources"
```
