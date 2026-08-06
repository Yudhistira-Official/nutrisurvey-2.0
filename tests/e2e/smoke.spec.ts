import { test, expect, type Page } from '@playwright/test';

const appUrl = process.env.NUTRISURVEY_SMOKE_URL || 'http://127.0.0.1:4173';
const mockedSecret = `acceptance-secret-${Date.now()}`;
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

async function installMockBridge(page: Page) {
  await page.addInitScript(({ fixtureFoods, secret }) => {
    type InvokePayload = { request?: unknown; bytes?: number[]; project?: unknown } | Uint8Array;
    type InvokeOptions = { headers?: Record<string, string> };
    const state = {
      calls: [] as Array<{ command: string; payloadKeys: string[] }>,
      serializedCalls: [] as string[],
      logs: [] as string[],
      rawSecretSeenOnlyInMemory: false,
      projectBytes: [] as number[],
      importedBytes: [] as number[],
    };
    const bridge = {
      invoke: async (command: string, payload?: InvokePayload, options?: InvokeOptions) => {
        const rawInvoke = JSON.stringify({ command, payload, options });
        if (rawInvoke.includes(secret)) state.rawSecretSeenOnlyInMemory = true;
        state.calls.push({ command, payloadKeys: payload && typeof payload === 'object' ? Object.keys(payload) : [] });
        state.serializedCalls.push(JSON.stringify({ command, payloadKeys: payload && typeof payload === 'object' ? Object.keys(payload) : [], optionKeys: options ? Object.keys(options) : [] }));
        state.logs.push(`invoke:${command}`);

        if (command === 'ping') return 'pong';
        if (command === 'food_status') return { foodCount: fixtureFoods.length, isReady: true };
        if (command === 'food_search') return fixtureFoods;
        if (command === 'nutrient_list') return [
          { name: 'energi', unit: 'kcal', amount: 130 },
          { name: 'protein', unit: 'g', amount: 2.7 },
        ];
        if (command === 'food_recommendations') return fixtureFoods;
        if (command === 'calculate_tdee') return {
          basalMetabolicRate: 1696,
          totalDailyEnergyExpenditure: 2035.2,
          formulaUsed: 'Harris-Benedict (Clinical Edition)',
          bmi: 22.86,
          nutritionClassification: 'Normal',
          idealWeight: 67.5,
          adjustedWeight: 68.13,
          referenceWeight: 70,
        };
        if (command === 'import_food_csv') {
          state.importedBytes = payload && !(payload instanceof Uint8Array) ? payload.bytes || [] : [];
          return 1;
        }
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
          bytes: [123, 92, 117, 53, 53, 51, 53, 54, 63],
          delivery: 'saved',
          savedPath: 'acceptance-report.rtf',
        };
        if (command === 'project_save') {
          const objectPayload = payload instanceof Uint8Array ? undefined : payload;
          state.projectBytes = Array.from(new TextEncoder().encode(JSON.stringify(objectPayload?.project)));
          return true;
        }
        if (command === 'project_open') {
          return JSON.parse(new TextDecoder().decode(Uint8Array.from(state.projectBytes)));
        }
        throw new Error(`Unexpected command: ${command}`);
      },
    };
    (window as Window & { __TAURI_INTERNALS__?: unknown; __NUTRISURVEY_SMOKE__?: unknown }).__TAURI_INTERNALS__ = bridge;
    (window as Window & { __NUTRISURVEY_SMOKE__?: unknown }).__NUTRISURVEY_SMOKE__ = { state, secret };
  }, { fixtureFoods: foods, secret: mockedSecret });
}

test('native UI smoke covers readiness, search, recommendations, import, roundtrip, TDEE, AI, and report export', async ({ page }) => {
  await installMockBridge(page);
  await page.goto(appUrl);
  await expect(page.getByRole('heading', { name: 'Manajemen Menu' })).toBeVisible();
  await expect(page.getByText('Database siap')).toBeVisible();

  await page.locator('tbody button').first().click();
  await page.getByPlaceholder('Ketik nama makanan...').fill('nasi');
  await page.getByRole('button', { name: /Nasi Fixture/ }).click();
  await expect(page.getByText('Nasi Fixture')).toBeVisible();

  await page.getByRole('button', { name: 'Rekomendasi' }).click();
  await page.getByRole('button', { name: 'Cari Rekomendasi' }).click();
  await expect(page.getByText('Nasi Fixture')).toBeVisible();
  await page.getByRole('button', { name: '+ Tambah' }).click();
  await expect(page.getByText('Daftar Konsumsi')).toBeVisible();

  await page.getByRole('button', { name: 'Impor CSV' }).click();
  await expect(page.getByText(/Database berhasil diimpor: 1 baris/)).toBeVisible();

  await page.getByRole('button', { name: 'Simpan Proyek' }).click();
  await expect(page.getByText('Proyek berhasil disimpan')).toBeVisible();
  page.once('dialog', dialog => dialog.accept());
  await page.getByRole('button', { name: 'Buka Proyek' }).click();
  await expect(page.getByText('Proyek berhasil dibuka')).toBeVisible();

  await page.getByRole('button', { name: 'Kalkulator TDEE' }).click();
  await page.getByRole('button', { name: 'Hitung & Terapkan' }).click();
  await expect(page.getByText('2035 kcal')).toBeVisible();

  await page.getByRole('button', { name: 'AI Meal Planner' }).click();
  await page.getByLabel('Model').fill('acceptance-model');
  await page.getByLabel('API Key').fill(mockedSecret);
  await page.getByRole('button', { name: 'Generate AI Meal Plan' }).click();
  await expect(page.getByText('Nasi Fixture')).toBeVisible();
  await page.getByRole('button', { name: /Implement/ }).click();

  await page.getByRole('button', { name: 'Word Report' }).click();
  await expect(page.getByText(/Laporan tersimpan/)).toBeVisible();
  const smokeState = await page.evaluate(() => (window as Window & { __NUTRISURVEY_SMOKE__?: { state: { calls: Array<{ command: string; payloadKeys: string[] }>; serializedCalls: string[]; logs: string[]; rawSecretSeenOnlyInMemory: boolean; projectBytes: number[]; importedBytes: number[] } } }).__NUTRISURVEY_SMOKE__?.state);
  expect(smokeState?.calls.map(call => call.command)).toEqual(expect.arrayContaining([
    'food_status',
    'food_search',
    'food_recommendations',
    'import_food_csv',
    'calculate_tdee',
    'generate_ai_menu',
    'export_word',
  ]));
  expect(smokeState?.projectBytes.length).toBeGreaterThan(0);
  expect(smokeState?.importedBytes.length).toBeGreaterThan(0);
  expect(new TextDecoder().decode(Uint8Array.from(smokeState?.projectBytes || []))).not.toContain(mockedSecret);
  expect(new TextDecoder().decode(Uint8Array.from(smokeState?.importedBytes || []))).not.toContain(mockedSecret);
  expect(smokeState?.rawSecretSeenOnlyInMemory).toBe(true);
  expect(JSON.stringify(smokeState?.calls || [])).not.toContain(mockedSecret);
  expect(JSON.stringify(smokeState?.serializedCalls || [])).not.toContain(mockedSecret);
  expect(JSON.stringify(smokeState?.logs || [])).not.toContain(mockedSecret);
  const bundleScan = await page.evaluate(async secret => {
    const urls = performance.getEntriesByType('resource').map(entry => (entry as PerformanceResourceTiming).name).filter(url => url.includes('/_next/'));
    const contents = await Promise.all(urls.map(async url => (await fetch(url)).text()));
    return { urls, contents: contents.map(content => ({ hasSecret: content.includes(secret), size: content.length })) };
  }, mockedSecret);
  expect(bundleScan.urls.length).toBeGreaterThan(0);
  expect(bundleScan.contents.every(content => !content.hasSecret && content.size > 0)).toBe(true);
});
