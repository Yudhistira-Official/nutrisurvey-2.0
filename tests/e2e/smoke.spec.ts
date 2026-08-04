import { test, expect } from '@playwright/test';

const appUrl = process.env.NUTRISURVEY_SMOKE_URL || 'http://127.0.0.1:4173';

const foods = [{
  id: 1,
  name: 'Nasi Fixture',
  brand: null,
  category: 'Pokok',
  servingSize: 100,
  servingUnit: 'g',
  servingsPerContainer: 1,
  nutrients: { energi: 130, protein: 2.7, 'karbohidrat total': 28.2, 'lemak total': 0.3 },
}];

async function installMockBridge(page: import('@playwright/test').Page) {
  await page.addInitScript(({ fixtureFoods }) => {
    const bridge = {
      invoke: async (command: string) => {
        if (command === 'ping') return 'pong';
        if (command === 'food_status') return { foodCount: fixtureFoods.length, isReady: true };
        if (command === 'food_search') return fixtureFoods;
        if (command === 'nutrient_list') return [
          { name: 'energi', unit: 'kcal', amount: 130 },
          { name: 'protein', unit: 'g', amount: 2.7 },
        ];
        if (command === 'calculate_tdee') return {
          basalMetabolicRate: 1696,
          totalDailyEnergyExpenditure: 2035.2,
          formulaUsed: 'Harris-Benedict (Clinical Edition)',
        };
        if (command === 'food_recommendations') return fixtureFoods;
        if (command === 'generate_ai_menu') return [{
          meal_type: 'Makan Pagi',
          requested_keyword: 'Nasi Fixture',
          matched_food_id: 1,
          matched_food_name: 'Nasi Fixture',
          suggested_grams: 100,
          reference_grams: 100,
          calories: 130,
          protein: 2.7,
          fat: 0.3,
          carbohydrate: 28.2,
          nutrients: { energi: 130 },
          reasoning: 'fixture',
        }];
        if (command === 'export_word') return {
          filename: 'Laporan_Nutrisi_acceptance.rtf',
          contentType: 'application/rtf',
          bytes: [123],
          delivery: 'saved',
          savedPath: 'acceptance-report.rtf',
        };
        throw new Error(`Unexpected command: ${command}`);
      },
    };
    (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = bridge;
  }, { fixtureFoods: foods });
}

test('native UI smoke covers readiness, search, TDEE, meal, mocked AI, and report export', async ({ page }) => {
  await installMockBridge(page);
  await page.goto(appUrl);
  await expect(page.getByRole('heading', { name: 'Manajemen Menu' })).toBeVisible();

  await page.getByRole('button', { name: 'Simpan Proyek' }).click();
  await page.getByRole('button', { name: 'Buka Proyek' }).click();
  await page.getByRole('button', { name: 'Impor CSV' }).click();
  await page.getByRole('button', { name: 'Word Report' }).click();

  await page.locator('tbody button').first().click();
  await page.getByPlaceholder('Ketik nama makanan...').fill('nasi');
  await page.getByRole('button', { name: /Nasi Fixture/ }).click();
  await expect(page.getByText('Nasi Fixture')).toBeVisible();

  await page.getByRole('button', { name: 'Kalkulator TDEE' }).click();
  await page.getByRole('button', { name: 'Hitung & Terapkan' }).click();
  await expect(page.getByText('2035 kcal')).toBeVisible();

  await page.getByRole('button', { name: 'AI Meal Planner' }).click();
  await page.getByLabel('Model').fill('acceptance-model');
  await page.getByLabel('API Key').fill('acceptance-secret');
  await page.getByRole('button', { name: 'Generate AI Meal Plan' }).click();
  await expect(page.getByText('Nasi Fixture')).toBeVisible();
  await page.getByRole('button', { name: /Implement/ }).click();

  await page.getByRole('button', { name: 'Word Report' }).click();
  await expect(page.getByText(/Laporan berhasil diekspor|Laporan tersimpan/)).toBeVisible();
});
