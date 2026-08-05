# TDEE Clinical Context for AI Meal Planner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Integrate calculated TDEE clinical data into AI Meal Planner and preserve the complete planner session when switching application tabs.

**Architecture:** Rust remains single source of truth for TDEE, BMI, diagnosis, and clinical reference weights. React stores the latest TDEE request/response plus macro targets in Home, passes a typed clinical context to AiMealPlanner, and includes it in every AI prompt. AiMealPlanner persists clinical context, assessment, chat, plan, and plan-start state in sessionStorage.

**Tech Stack:** React 19, Next.js 16, TypeScript, Tauri v2, Rust, existing SQLite/TDEE command.

## Global Constraints

- Do not read or commit `.env`.
- Do not expose API keys in frontend state persistence or prompts.
- Do not duplicate BMI/diagnosis calculations in React.
- Do not commit or push changes.
- Preserve existing AI provider routing and Dashboard plan application.
- Keep all existing tests passing and add focused regression coverage.

---

### Task 1: Define clinical context contract

**Files:**
- Modify: `src/lib/types.ts`
- Modify: `src/components/TdeeCalculator.tsx`
- Modify: `src/app/page.tsx`
- Test: `tests/components.test.mjs`

**Interfaces:**
- Produce `TdeeClinicalContext` containing `request`, `assessment`, and `targets`.
- `TdeeCalculator` calls `onClinicalContext(context)` after successful calculation.
- `Home` stores `tdeeContext: TdeeClinicalContext | null` and passes it to `AiMealPlanner`.

- [ ] Add exact type:

```ts
export type TdeeClinicalContext = {
  request: TdeeRequest;
  assessment: TdeeResponse;
  targets: Targets;
};
```

- [ ] Change `TdeeCalculator` props to accept `onClinicalContext: (context: TdeeClinicalContext) => void` while retaining `onTargets` only if existing consumers require it.
- [ ] Call `onClinicalContext` with the exact request submitted, returned Rust assessment, and calculated macro targets.
- [ ] Add `tdeeContext` state in `Home`; update it from `TdeeCalculator` and restore it from project data only if the project format already contains compatible TDEE data.
- [ ] Pass `tdeeContext` to `AiMealPlanner`.
- [ ] Add source-level test asserting TDEE response fields are forwarded to planner context.
- [ ] Run `npm test` and confirm failure/pass for this contract.

### Task 2: Integrate clinical context into AI Planner

**Files:**
- Modify: `src/components/AiMealPlanner.tsx`
- Modify: `src-tauri/src/ai/prompt.rs`
- Test: `tests/components.test.mjs`
- Test: `src-tauri/tests/ai.rs`

**Interfaces:**
- `AiMealPlanner` consumes `clinicalContext: TdeeClinicalContext | null`.
- Prompt receives serialized clinical facts: gender, weight, height, age, BMI standard, BMI, nutrition classification, ideal/adjusted/reference weight, BMR, TDEE, AF, IF, and macro targets.

- [ ] Block plan generation when `clinicalContext` is null with message `Hitung TDEE dan diagnosis gizi terlebih dahulu.`
- [ ] Add a compact clinical summary card to the left planner panel showing BMI, classification, TDEE, and reference weight.
- [ ] Build a prompt section from the typed context; do not calculate or classify in React.
- [ ] Update Rust `prompt::build` wording so clinical context is authoritative and the AI must use classification-specific rules from NutrisionistInstruction.
- [ ] Ensure prompts explicitly distinguish normal, overweight/obesity, malnutrition, and other classifications while requiring assessment data to remain user-provided.
- [ ] Add Rust prompt test assertions for BMI, classification, weight, and TDEE values.
- [ ] Run frontend and Rust AI tests.

### Task 3: Persist planner and TDEE context across tab changes

**Files:**
- Modify: `src/components/AiMealPlanner.tsx`
- Modify: `src/app/page.tsx`
- Test: `tests/components.test.mjs`

**Interfaces:**
- Session key remains `nutrisurvey.ai-planner-session` and now stores `clinicalContext`, `rows`, `chat`, `assessmentAnswers`, `assessmentVisible`, and `planStarted`.

- [ ] Serialize `clinicalContext` into sessionStorage without config or API key fields.
- [ ] Restore planner state and clinical context on mount before rendering plan/chat controls.
- [ ] Keep `planStarted`, assessment answers, plan rows, and chat stable when `section` changes from `ai` to another tab and back.
- [ ] Update context when a new TDEE calculation succeeds, while preserving chat history unless the user starts a fresh session.
- [ ] Add test coverage for session payload excluding `apiKey` and retaining clinical fields.
- [ ] Run `npm test`, `npx tsc --noEmit`, and `npm run lint`.

### Task 4: Verify integration and regression safety

**Files:**
- Modify: `tests/components.test.mjs` only if additional regression assertions are needed.

- [ ] Run `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`.
- [ ] Run `cargo check --manifest-path src-tauri/Cargo.toml`.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1`.
- [ ] Run `npm test`.
- [ ] Run `npx tsc --noEmit`.
- [ ] Run `npm run lint`.
- [ ] Run `git diff --check`.
- [ ] Inspect `git diff` to confirm only intended files changed and no secrets are present.
