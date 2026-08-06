import test from 'node:test';
import assert from 'node:assert/strict';
import { calculateTotals, calculateMacroTargets, implementAiRows, moveFoodToMeal, moveFoodToMealAtPosition, parseProject, reorderFoods, reorderItems, serializeProject } from '../src/lib/types.ts';
import { createCommandAdapters } from '../src/lib/commands.ts';
import { classifyUiError } from '../src/lib/types.ts';
import { createProjectFileAdapter } from '../src/lib/project.ts';
import { readFileSync, mkdtempSync, writeFileSync, readFileSync as readProjectFile, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

test('tdee calculator shows Indonesian gender labels', () => {
  const source = readFileSync(new URL('../src/components/TdeeCalculator.tsx', import.meta.url), 'utf8');
  assert.match(source, /value="Male">Laki-laki/);
  assert.match(source, /value="Female">Perempuan/);
});

test('tdee calculator uses responsive paired panels and AF IF labels', () => {
  const source = readFileSync(new URL('../src/components/TdeeCalculator.tsx', import.meta.url), 'utf8');
  assert.match(source, /tdee-panels/);
  assert.match(source, /Faktor Aktivitas \(AF\)/);
  assert.match(source, /Faktor Cedera \(IF\)/);
  assert.match(source, /Asia Pasifik/);
  assert.match(source, /WHO Internasional/);
  assert.match(source, /nutrition-standard/);
  assert.match(source, /diagnosis-summary/);
  assert.match(source, /Rawat Inap/);
  assert.match(source, /Pneumonia/);
  assert.match(source, /Input Manual/);
  assert.match(source, /isManualFactors \? <input/);
});

test('settings project history uses in-app project opener while reports use external opener', () => {
  const source = readFileSync(new URL('../src/components/SettingsPanel.tsx', import.meta.url), 'utf8');
  assert.match(source, /onOpenProjectPath/);
  assert.match(source, /item\.kind === 'project'/);
  assert.match(source, /openExistingFile/);
});

test('project open flow confirms before replacing dashboard state', () => {
  const source = readFileSync(new URL('../src/app/page.tsx', import.meta.url), 'utf8');
  assert.match(source, /Buka project ini/);
  assert.match(source, /Data Dashboard saat ini akan diganti/);
  assert.match(source, /openProjectPath/);
});

test('settings panels stretch to viewport height and equal height', () => {
  const css = readFileSync(new URL('../src/styles/globals.css', import.meta.url), 'utf8');
  assert.match(css, /settings-page-grid[^}]*align-items:stretch/);
  assert.match(css, /settings-page-grid>\.card[^}]*height:100%/);
  assert.match(css, /min-height:calc\(100vh/);
});

test('toast notifications auto-dismiss after three seconds with exit animation', () => {
  const source = readFileSync(new URL('../src/app/page.tsx', import.meta.url), 'utf8');
  const css = readFileSync(new URL('../src/styles/globals.css', import.meta.url), 'utf8');
  assert.match(source, /toastVisible/);
  assert.match(source, /3000/);
  assert.match(source, /toastVisible \? 'toast-visible' : 'toast-exiting'/);
  assert.match(css, /@keyframes toastEnter/);
  assert.match(css, /@keyframes toastExit/);
});

test('dashboard totals preserve serving scaling and targets', () => {
  const foods = [{
    id: 'food-1', name: 'Rice', servingSize: 100, servingUnit: 'g', servingsPerContainer: 1,
    amount: 150, mealTime: 'LUNCH', nutrients: { energi: 130, 'karbohidrat total': 28, protein: 2.7, 'lemak total': 0.3 },
  }];
  const result = calculateTotals(foods, { kcal: 500, carbs: 100, protein: 40, fat: 20 });
  assert.equal(result.energy, 195);
  assert.equal(result.carbs, 42);
  assert.ok(Math.abs(result.protein - 4.05) < 0.000001);
  assert.ok(Math.abs(result.fat - 0.45) < 0.000001);
  assert.deepEqual(result.targets, { kcal: 500, carbs: 100, protein: 40, fat: 20 });
});

test('versioned project round trip rejects unsupported versions', () => {
  const project = { foods: [], meals: [{ id: 'BREAKFAST', label: 'Makan Pagi' }], targets: { kcal: 0, carbs: 0, protein: 0, fat: 0 } };
  assert.deepEqual(parseProject(serializeProject(project)), { version: 1, ...project });
  assert.throws(() => parseProject(JSON.stringify({ ...project, version: 2 })), /tidak didukung/);
  assert.throws(() => parseProject(JSON.stringify({ version: 1, foods: [{ id: 'x' }], meals: project.meals, targets: project.targets })), /tidak valid/);
});

test('project file adapter delegates selected-path handling to native commands', async () => {
  const calls = [];
  const project = { foods: [], meals: [{ id: 'BREAKFAST', label: 'Makan Pagi' }], targets: { kcal: 0, carbs: 0, protein: 0, fat: 0 } };
  const adapter = createProjectFileAdapter({
    saveProject: async value => { calls.push(['save', value]); return true; },
    openProject: async () => { calls.push(['open']); return { version: 1, ...project }; },
  });
  assert.equal(await adapter.save(project), true);
  assert.deepEqual(await adapter.open(), { version: 1, ...project });
  assert.deepEqual(calls, [['save', { version: 1, ...project }], ['open']]);
});

test('real .nutri file write, read, and re-import preserves project data', () => {
  const project = {
    foods: [{ id: 'fixture-1', name: 'Nasi 🍚', servingSize: 100, servingUnit: 'g', servingsPerContainer: 1, amount: 125, mealTime: 'BREAKFAST', nutrients: { energi: 130 } }],
    meals: [{ id: 'BREAKFAST', label: 'Makan Pagi' }],
    targets: { kcal: 2000, carbs: 250, protein: 100, fat: 60 },
  };
  const directory = mkdtempSync(join(tmpdir(), 'nutrisurvey-nutri-'));
  const path = join(directory, 'fixture.nutri');
  try {
    writeFileSync(path, serializeProject(project), 'utf8');
    const imported = parseProject(readProjectFile(path, 'utf8'));
    assert.deepEqual(imported, { version: 1, ...project });
    assert.equal(imported.foods[0].name, 'Nasi 🍚');
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test('food search uses selected row and explicit add action', () => {
  const source = readFileSync(new URL('../src/components/FoodSearch.tsx', import.meta.url), 'utf8');
  assert.match(source, /selectedFoodId/);
  assert.match(source, /className={`search-result-row/);
  assert.match(source, /\+ Tambah/);
  assert.match(source, /food-search-inputs/);
  assert.match(source, /selectedFood\) add\(selectedFood\)/);
  assert.equal((source.match(/className="gram-input"/g) || []).length, 1);
});

test('search adapter forwards query and limit', async () => {
  const calls = [];
  const adapters = createCommandAdapters(async (command, payload) => { calls.push({ command, payload }); return []; });
  await adapters.searchFoodsByName('rice');
  assert.deepEqual(calls[0], { command: 'food_search', payload: { query: 'rice', limit: 20 } });
});

test('tdee validation blocks invalid percentages and preserves manual factors', () => {
  assert.equal(calculateMacroTargets({ totalDailyEnergyExpenditure: 2000 }, { carbs: 40, protein: 30, fat: 20 }), null);
  const request = { gender: 'Male', weightKg: 70, heightCm: 170, age: 25, activityFactor: 1.2, injuryFactor: 1, isManualFactors: true };
  assert.deepEqual(request.isManualFactors, true);
});

test('recommendation add and drag move food to selected meal', () => {
  const food = { id: 7, name: 'Rice', servingSize: 100, servingUnit: 'g', servingsPerContainer: 1, nutrients: {} };
  const added = { ...food, id: 'session-1', amount: 100, mealTime: 'BREAKFAST' };
  assert.equal(added.mealTime, 'BREAKFAST');
  assert.equal(moveFoodToMeal([added], 'session-1', 'DINNER')[0].mealTime, 'DINNER');
});

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

test('AI preview implementation maps duplicate meal labels by occurrence', () => {
  const rows = [
    { meal_type: 'Selingan', requested_keyword: 'apple', matched_food_id: 7, matched_food_name: 'Apple', suggested_grams: 120, calories: 60, protein: 0, fat: 0, carbohydrate: 15, nutrients: {}, reasoning: 'fit' },
    { meal_type: 'Selingan', requested_keyword: 'banana', matched_food_id: 8, matched_food_name: 'Banana', suggested_grams: 100, calories: 89, protein: 1, fat: 0, carbohydrate: 23, nutrients: {}, reasoning: 'fit' },
  ];
  const result = implementAiRows(rows, [{ id: 'SNACK-1', label: 'Selingan' }, { id: 'SNACK-2', label: 'Selingan' }]);
  assert.deepEqual(result.added.map(food => food.mealTime), ['SNACK-1', 'SNACK-2']);
});

test('meal and food reorder use stable internal IDs', () => {
  const meals = [{ id: 'SNACK-1', label: 'Selingan' }, { id: 'SNACK-2', label: 'Selingan' }, { id: 'DINNER', label: 'Makan Malam' }];
  assert.deepEqual(reorderItems(meals, 'SNACK-2', 'DINNER').map(meal => meal.id), ['SNACK-1', 'DINNER', 'SNACK-2']);
  const foods = [
    { id: 'food-1', name: 'Apple', servingSize: 100, servingUnit: 'g', servingsPerContainer: 1, amount: 100, mealTime: 'SNACK-1', nutrients: {} },
    { id: 'food-2', name: 'Pear', servingSize: 100, servingUnit: 'g', servingsPerContainer: 1, amount: 100, mealTime: 'SNACK-1', nutrients: {} },
  ];
  assert.deepEqual(reorderFoods(foods, 'food-2', 'food-1').map(food => food.id), ['food-2', 'food-1']);
});

test('project import rejects duplicate meal IDs while allowing duplicate labels', () => {
  const project = { foods: [], meals: [{ id: 'SNACK-1', label: 'Selingan' }, { id: 'SNACK-2', label: 'Selingan' }], targets: { kcal: 0, carbs: 0, protein: 0, fat: 0 } };
  assert.deepEqual(parseProject(serializeProject(project)).meals.map(meal => meal.label), ['Selingan', 'Selingan']);
  assert.throws(() => parseProject(serializeProject({ ...project, meals: [{ id: 'SAME', label: 'Selingan' }, { id: 'SAME', label: 'Selingan' }] })), /tidak valid/);
});

test('report adapter invokes export command', async () => {
  const calls = [];
  const adapters = createCommandAdapters(async (command, payload) => { calls.push({ command, payload }); return { filename: 'report.rtf' }; });
  await adapters.exportToWord({ foods: [], mealTimes: [], targets: { kcal: 0, carbs: 0, protein: 0, fat: 0 } });
  assert.equal(calls[0].command, 'export_word');
});

test('native rejection paths become safe user messages', () => {
  assert.equal(classifyUiError({ kind: 'Export', message: 'cancelled' }, 'Export'), 'Ekspor dibatalkan');
  assert.equal(classifyUiError({ kind: 'Validation', message: 'bad data' }, 'Export'), 'Data tidak valid');
  assert.equal(classifyUiError({ kind: 'Io', message: 'write failed' }, 'Export'), 'Operasi file gagal');
});

test('responsive stylesheet defines mobile sidebar and grid layout', () => {
  const css = readFileSync(new URL('../src/styles/globals.css', import.meta.url), 'utf8');
  assert.match(css, /@media\s*\(max-width:\s*900px\)/);
  assert.match(css, /\.sidebar[^}]*width:100%/);
  assert.match(css, /\.dashboard-grid[^}]*grid-template-columns:1fr/);
  assert.match(css, /\.ai-grid[^}]*grid-template-columns:1fr/);
});

test('AI streaming path emits incremental chunks for non-streaming providers', () => {
  const source = readFileSync(new URL('../src-tauri/src/ai/mod.rs', import.meta.url), 'utf8');
  assert.match(source, /send_stream_chunks\(&channel, &content\)/);
  assert.match(source, /chars\.chunks\(48\)/);
  assert.match(source, /channel\.send\(chunk\.iter\(\)\.collect\(\)\)/);
});

test('New Chat cancels active AI request and invalidates stale lifecycle', () => {
  const source = readFileSync(new URL('../src/components/AiMealPlanner.tsx', import.meta.url), 'utf8');
  const commands = readFileSync(new URL('../src/lib/commands.ts', import.meta.url), 'utf8');
  assert.match(source, /cancelAiRequest/);
  assert.match(source, /activeRequestId/);
  assert.match(commands, /cancel_ai_request/);
});

test('AI stream failure creates one retryable error bubble', () => {
  const source = readFileSync(new URL('../src/components/AiMealPlanner.tsx', import.meta.url), 'utf8');
  assert.match(source, /const errorId = chatId\(\)/);
  assert.match(source, /current\.filter\(entry => entry\.id !== streamId\), \{ id: errorId/);
  assert.doesNotMatch(source, /else \{\s*setChat\(current => \[\.\.\.current, \{ id: chatId\(\), role: 'assistant' as const, content: message/);
});

test('AI failures keep retryable error bubble visible after stream failure', () => {
  const source = readFileSync(new URL('../src/components/AiMealPlanner.tsx', import.meta.url), 'utf8');
  assert.match(source, /onNotify\(/);
  assert.match(source, /formatAiError/);
  assert.match(source, /isError: true, retryPrompt: instruction/);
  assert.match(source, /setChat\(current => current\.map\(entry => entry\.id === errorId/);
});

test('AI generation requests database-backed verification with bounded retries', () => {
  const source = readFileSync(new URL('../src/components/AiMealPlanner.tsx', import.meta.url), 'utf8');
  const commands = readFileSync(new URL('../src/lib/commands.ts', import.meta.url), 'utf8');
  assert.match(source, /verifyMenu: true/);
  assert.match(commands, /candidateCatalog/);
  assert.match(commands, /verifyMenu/);
});

test('AI revisions send active menu context and explicit deletion rules', () => {
  const source = readFileSync(new URL('../src/components/AiMealPlanner.tsx', import.meta.url), 'utf8');
  const prompt = readFileSync(new URL('../src-tauri/src/ai/prompt.rs', import.meta.url), 'utf8');
  assert.match(source, /activeMenu/);
  assert.match(source, /revision/);
  assert.match(prompt, /Menu aktif yang wajib dipertahankan/);
  assert.match(prompt, /hapus|menghapus/);
});

test('dashboard exposes clear drag and drop targets for meal sections', () => {
  const source = readFileSync(new URL('../src/components/Dashboard.tsx', import.meta.url), 'utf8');
  assert.match(source, /draggable/);
  assert.match(source, /drop-zone|drop target/i);
  assert.match(source, /onDragEnd/);
});

test('stream parser accepts data lines without a space and flushes final chunk', () => {
  const source = readFileSync(new URL('../src-tauri/src/ai/openai.rs', import.meta.url), 'utf8');
  assert.match(source, /strip_prefix\("data:"\)/);
  assert.match(source, /buf\.trim\(\)\.is_empty\(\)/);
});

test('new chat atomically clears persisted history and invalidates stale requests', () => {
  const source = readFileSync(new URL('../src/components/AiMealPlanner.tsx', import.meta.url), 'utf8');
  assert.match(source, /sessionStorage\.removeItem\(storageKey\)/);
  assert.match(source, /requestGeneration/);
  assert.match(source, /setChat\(\[\{ id: chatId\(\), role: 'assistant'/);
});

test('AI generation resets streaming state after failure so retry is clickable', () => {
  const source = readFileSync(new URL('../src/components/AiMealPlanner.tsx', import.meta.url), 'utf8');
  assert.match(source, /setStreamingId\(null\)/);
  assert.match(source, /setStreamingId\(null\)/);
  assert.match(source, /isError: true, retryPrompt: instruction/);
  assert.match(source, /retryMessage/);
  assert.match(source, /disabled=\{loading \|\| streamingId !== null\}/);
});
