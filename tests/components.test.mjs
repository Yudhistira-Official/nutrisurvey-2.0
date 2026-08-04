import test from 'node:test';
import assert from 'node:assert/strict';
import { calculateTotals, calculateMacroTargets, implementAiRows, moveFoodToMeal, parseProject, serializeProject } from '../src/lib/types.ts';
import { createCommandAdapters } from '../src/lib/commands.ts';
import { classifyUiError } from '../src/lib/types.ts';
import { createProjectFileAdapter } from '../src/lib/project.ts';
import { readFileSync, mkdtempSync, writeFileSync, readFileSync as readProjectFile, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

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

test('project file adapter uses dialog and filesystem boundaries for round trip', async () => {
  let bytes;
  const project = { foods: [], meals: [{ id: 'BREAKFAST', label: 'Makan Pagi' }], targets: { kcal: 0, carbs: 0, protein: 0, fat: 0 } };
  const adapter = createProjectFileAdapter(
    { save: async () => '/tmp/fixture.nutri', open: async () => '/tmp/fixture.nutri' },
    { writeFile: async (_path, value) => { bytes = value; }, readFile: async () => bytes },
  );
  assert.equal(await adapter.save(project), true);
  assert.deepEqual(await adapter.open(), { version: 1, ...project });
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

test('AI preview implementation maps matched rows to dashboard meals', () => {
  const rows = [{ meal_type: 'Makan Pagi', requested_keyword: 'rice', matched_food_id: 7, matched_food_name: 'Rice', suggested_grams: 120, calories: 156, protein: 3, fat: 1, carbohydrate: 34, nutrients: {}, reasoning: 'fit' }];
  const result = implementAiRows(rows, [{ id: 'BREAKFAST', label: 'Makan Pagi' }]);
  assert.equal(result.added.length, 1);
  assert.equal(result.added[0].mealTime, 'BREAKFAST');
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
