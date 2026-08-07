# Dashboard Drag and Word Report Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make cross-meal food drag-and-drop reliable and align Word/RTF report calculation columns while removing the unwanted slash from the diet-results heading.

**Architecture:** Keep React state as source of truth. Add a pure cross-meal insertion helper in `src/lib/types.ts`, use it from Dashboard callbacks, and stabilize native drag event handling in `Dashboard.tsx`. Keep report generation in the native Rust renderer, using one shared tab-stop layout for header and rows and explicit four-column row output.

**Tech Stack:** Next.js/React/TypeScript, Node test runner, Rust/Tauri, RTF template.

## Global Constraints

- Do not change export request or Tauri command interfaces.
- Invalid drag payloads and unknown IDs must leave state unchanged.
- Cross-meal drop on meal zone appends to destination meal; drop on food row inserts at target row.
- The first report heading must be exactly `HASIL PERHITUNGAN DIET`; it must not contain `/`.
- Preserve RTF escaping and invalid-template validation.
- Do not modify unrelated existing work in the working tree.

---

### Task 1: Add tested cross-meal food insertion behavior

**Files:**
- Modify: `src/lib/types.ts:186-209`
- Modify: `tests/components.test.mjs:136-160`

**Interfaces:**
- Consumes: `SessionFood[]`, dragged food ID, target food ID or destination meal ID.
- Produces: `moveFoodToMeal`, `reorderFoods`, and a new exported `moveFoodToMealAtPosition(foods, draggedId, mealId, targetId?)` pure helper returning `SessionFood[]`.

- [ ] **Step 1: Write failing tests**

Add tests covering:

```js
test('cross-meal food drop appends to destination meal', () => {
  const foods = [
    { id: 'food-1', name: 'Rice', amount: 100, servingSize: 100, servingUnit: 'g', servingsPerContainer: 1, mealTime: 'BREAKFAST', nutrients: {} },
    { id: 'food-2', name: 'Egg', amount: 50, servingSize: 100, servingUnit: 'g', servingsPerContainer: 1, mealTime: 'DINNER', nutrients: {} },
  ];
  assert.deepEqual(moveFoodToMeal(foods, 'food-1', 'DINNER').map(food => [food.id, food.mealTime]), [
    ['food-2', 'DINNER'],
    ['food-1', 'DINNER'],
  ]);
});

test('cross-meal food drop on row inserts at target position', () => {
  const foods = [
    { id: 'food-1', name: 'Rice', amount: 100, servingSize: 100, servingUnit: 'g', servingsPerContainer: 1, mealTime: 'BREAKFAST', nutrients: {} },
    { id: 'food-2', name: 'Egg', amount: 50, servingSize: 100, servingUnit: 'g', servingsPerContainer: 1, mealTime: 'DINNER', nutrients: {} },
    { id: 'food-3', name: 'Milk', amount: 200, servingSize: 100, servingUnit: 'ml', servingsPerContainer: 1, mealTime: 'DINNER', nutrients: {} },
  ];
  assert.deepEqual(moveFoodToMealAtPosition(foods, 'food-1', 'DINNER', 'food-3').map(food => [food.id, food.mealTime]), [
    ['food-2', 'DINNER'],
    ['food-1', 'DINNER'],
    ['food-3', 'DINNER'],
  ]);
});
```
Import the new helper in the test import list.

- [ ] **Step 2: Run focused tests and verify failure**

Run: `node --test tests/components.test.mjs`
Expected: FAIL because cross-meal append currently leaves the moved item in its original global position and the insertion helper does not exist.

- [ ] **Step 3: Implement minimal pure state helpers**

Change `moveFoodToMeal` to remove the matching food from its current array position, update its `mealTime`, and append it after all foods currently assigned to `mealTime`. Return the original array when either ID is missing. Add `moveFoodToMealAtPosition` that removes the dragged item, updates its meal, finds `targetId` in the remaining array, and inserts before target; if target is missing or belongs to another meal, return the original array. Keep same-meal ordering behavior in `reorderFoods` unchanged.

- [ ] **Step 4: Run focused tests and verify pass**

Run: `node --test tests/components.test.mjs`
Expected: PASS.

### Task 2: Stabilize Dashboard drag event routing

**Files:**
- Modify: `src/components/Dashboard.tsx:3-38`
- Modify: `src/app/page.tsx:50-61,92`
- Modify: `tests/components.test.mjs:220-225`

**Interfaces:**
- Consumes: drag payload type/ID and target meal/food IDs.
- Produces: validated callbacks with append-to-meal and insert-on-row semantics.

- [ ] **Step 1: Add source-level regression assertions**

Extend the Dashboard source test to require `application/x-nutrisurvey-type`, `getData`, and a guard for both food and meal IDs. Add a direct unit assertion for `moveFoodToMealAtPosition` behavior already introduced in Task 1.

- [ ] **Step 2: Run focused test and verify failure if assertions are absent**

Run: `node --test tests/components.test.mjs`
Expected: PASS for existing checks and fail only for any newly added assertion not yet represented by source.

- [ ] **Step 3: Implement validated and stable event handling**

In `Dashboard.tsx`:
- Import `moveFoodToMealAtPosition` only if callback logic is kept in component; otherwise keep state logic in page and route target IDs through callback.
- Add a helper that reads both drag type and ID and accepts only `meal` or `food` with nonempty ID.
- In `dropOnMeal`, invoke `onMoveFood` only for valid food payloads and `onReorderMeals` only for valid meal payloads.
- In `dropOnFood`, invoke a new callback signature `(draggedId, targetId)` that allows page state to choose same-meal reorder versus cross-meal insertion.
- Remove `onDragLeave` handlers from meal zones and food rows so child transitions cannot clear active drop state; clear state in `dropOnMeal`, `dropOnFood`, and `finishDrag`.
- Ensure food rows call `stopPropagation` on drop and meal zones still receive food drops on empty areas.

In `page.tsx`, update the food-row callback to call `reorderFoods` for same-meal IDs and `moveFoodToMealAtPosition` with the target food's `mealTime` for cross-meal IDs. Update Dashboard prop typing accordingly.

- [ ] **Step 4: Run frontend tests and typecheck**

Run: `node --test tests/components.test.mjs && npm run typecheck`
Expected: PASS with no TypeScript errors.

### Task 3: Align RTF calculation columns and remove slash

**Files:**
- Modify: `src-tauri/src/export.rs:140-180,264-374`
- Modify: `Assets/template.rtf:45-80`
- Modify: `src-tauri/tests/export.rs:36-53`

**Interfaces:**
- Consumes: existing `ExportRequest` and bundled RTF template.
- Produces: valid RTF bytes with stable four-column report rows and exact heading text.

- [ ] **Step 1: Add failing export assertions**

Extend `bundled_template_renders_expected_report_sections` with:

```rust
assert!(!output.contains("HASIL PERHITUNGAN DIET/"));
assert!(output.contains("\\tqc\\tx3000\\tqc\\tx5700\\tqc\\tx8400"));
assert!(output.contains("energy\\tab 200,0 kcal\\tab 2000,0 kcal\\tab 10 %"));
assert!(output.contains("protein\\tab 0,0 g(0%)\\tab 50,0 g\\tab 0 %"));
```

Also assert source template contains the clean heading and no `DIET/` text via a small test using `include_str!`.

- [ ] **Step 2: Run native export tests and verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test export`
Expected: FAIL on the slash assertion and/or inconsistent generated heading/column output.

- [ ] **Step 3: Implement minimal RTF layout fix**

In `render_rtf`:
- Change the generated first heading literal from `HASIL PERHITUNGAN DIET/` to `HASIL PERHITUNGAN DIET`.
- Use `\\tqc\\tx3000\\tqc\\tx5700\\tqc\\tx8400` for both the calculation header and data rows (or define one local layout string and reuse it).
- Ensure `nutrient_rows` emits exactly four fields separated by `\\tab`: nutrient name, analysis, recommendation, percentage. Do not embed recommendation annotations such as `< 30 %` or `> 55 %` in the recommendation field unless they are part of the existing intended copy; preserve current numeric semantics.
- Keep analysis macro percentage inside analysis field, recommendation value in recommendation field, and fulfillment percentage in fourth field.

In `Assets/template.rtf`, replace any `HASIL PERHITUNGAN DIET/` text with `HASIL PERHITUNGAN DIET` and align its calculation-section tab stops with `3000/5700/8400`.

- [ ] **Step 4: Run native export tests and formatting checks**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test export && cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
Expected: PASS.

### Task 4: Full verification and review

**Files:**
- No additional files expected.

- [ ] **Step 1: Run frontend verification**

Run: `npm run test:compile && node --test tests/**/*.test.mjs && npm run typecheck && npm run lint`
Expected: all commands PASS.

- [ ] **Step 2: Run native verification**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test export && cargo test --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
Expected: all commands PASS.

- [ ] **Step 3: Inspect diff and working tree**

Run: `git diff --check && git status --short`
Expected: no whitespace errors; only intended source, test, template, and plan/spec files changed. Preserve unrelated pre-existing modifications.
