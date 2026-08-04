import test from 'node:test';
import assert from 'node:assert/strict';
import { calculateTotals, parseProject, serializeProject } from '../src/lib/types.ts';

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
});
