
## Review findings resolved

- Restored HTML5 meal drag/drop; dropping food updates persisted `mealTime` through shared state and `.nutri` serialization.
- Added `isManualFactors` to typed TDEE request and native invoke payload.
- Save/open/CSV import/report actions now distinguish cancellation, native errors, validation errors, and successful completion without mutating state on invalid import.
- Deep `.nutri` v1 validation now checks food records, meal records, meal references, nutrient maps, targets, and finite numeric values before state update.
- Added native CSV dialog/read/import action to navigation.
- Added behavior coverage for search, TDEE validation, recommendation add/move, AI preview implementation, and report invocation; suite now has 9 passing tests.
- Made npm test perform clean-tree TypeScript validation before Node tests.
- Removed desktop `minWidth` restriction for mobile-safe resizing.

## Final review verification

- `npm test` passed: 9 tests.
- `rm -rf .next && npm run typecheck` passed.
- `npm run build` passed with static `/` and `/_not-found` routes.
- `cargo check --manifest-path src-tauri/Cargo.toml` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed all Rust tests.
