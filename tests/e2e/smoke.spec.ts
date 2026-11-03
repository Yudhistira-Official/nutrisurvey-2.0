import { test, expect } from './fixtures';
import type { Page } from '@playwright/test';

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
  await page.addInitScript(({ fixtureFoods, fixtureCsvPath, secret }) => {
    type InvokePayload = { request?: unknown; bytes?: number[]; project?: unknown } | Uint8Array;
    type InvokeOptions = { headers?: Record<string, string> };
    const state = {
      calls: [] as Array<{ command: string; payloadKeys: string[] }>,
      serializedCalls: [] as string[],
      logs: [] as string[],
      rawSecretSeenOnlyInMemory: false,
      projectBytes: [] as number[],
      importedBytes: [] as number[],
      aiKey: '' as string,
    };
    // Confirm browser tidak didukung semua driver CDP (mis. Lightpanda):
    // stub agar alur Buka Proyek deterministik di e2e.
    window.confirm = () => true;
    const bridge = {
      // Stub transformCallback untuk Channel @tauri-apps/api (streaming).
      transformCallback: () => 0,
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
        // Plugin dialog: kembalikan path fixture agar alur impor CSV berjalan
        // seperti di aplikasi nyata tanpa membuka dialog OS.
        if (command === 'plugin:dialog|open') return fixtureCsvPath;
        if (command === 'import_csv_from_path') {
          state.importedBytes = Array.from(new TextEncoder().encode(fixtureCsvPath));
          return 1;
        }
        if (command === 'ai_key_save') { state.aiKey = String((payload as { value?: string } | undefined)?.value ?? ''); return null; }
        if (command === 'ai_key_load') return state.aiKey || null;
        if (command === 'ai_key_delete') { state.aiKey = ''; return null; }
        if (command === 'ai_default_info') return { available: false, provider: 'openrouter', model: '', baseUrl: '' };
        // Streaming tidak didukung mock: lempar error yang memicu fallback generate_ai_menu.
        if (command === 'stream_ai_menu') throw new Error('network unavailable in mock bridge');
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
        if (command === 'export_word' || command === 'export_report') return {
          filename: command === 'export_report' ? 'Laporan_Nutrisi_acceptance.pdf' : 'Laporan_Nutrisi_acceptance.rtf',
          contentType: command === 'export_report' ? 'application/pdf' : 'application/rtf',
          bytes: [123, 92, 117, 53, 53, 51, 53, 54, 63],
          delivery: 'saved',
          savedPath: command === 'export_report' ? 'acceptance-report.pdf' : 'acceptance-report.rtf',
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
  }, { fixtureFoods: foods, fixtureCsvPath: '/tmp/nutrisurvey-fixture.csv', secret: mockedSecret });
}

test('native UI smoke covers readiness, search, recommendations, import, roundtrip, TDEE, AI, and report export', async ({ page }) => {
  await installMockBridge(page);
  await page.goto(appUrl);
  await expect(page.getByRole('heading', { name: 'Manajemen Menu' })).toBeVisible();
  await expect(page.getByText('Database siap')).toBeVisible();

  // Popup donasi menutup UI dengan overlay; tutup dulu agar klik berikutnya tidak terblokir.
  const donation = page.getByRole('dialog', { name: 'Donasi' });
  if (await donation.isVisible()) {
    await donation.getByRole('button').click();
    await expect(donation).toBeHidden();
  }

  await page.locator('tbody button').first().click();
  await page.getByPlaceholder('Ketik nama makanan...').fill('nasi');
  await page.getByRole('button', { name: /Nasi Fixture/ }).click();
  await expect(page.getByText('Nasi Fixture')).toBeVisible();

  await page.getByRole('button', { name: 'Rekomendasi' }).click();
  await page.getByRole('button', { name: 'Cari Makanan' }).click();
  await expect(page.getByText('Nasi Fixture').first()).toBeVisible();
  await page.getByRole('article').getByRole('button', { name: '+ Tambah' }).first().click();
  await expect(page.getByText('Daftar Konsumsi')).toBeVisible();

  await page.getByRole('button', { name: 'Impor CSV' }).click();
  await expect(page.getByText(/Database berhasil diimpor: 1 baris/)).toBeVisible();

  await page.getByRole('button', { name: 'Simpan Proyek' }).click();
  await expect(page.getByText('Proyek berhasil disimpan')).toBeVisible();
  await page.getByRole('button', { name: 'Buka Proyek' }).click();
  await expect(page.getByText('Proyek berhasil dibuka')).toBeVisible();

  await page.getByRole('button', { name: 'Kalkulator TDEE' }).click();
  await page.getByRole('button', { name: 'Hitung & Terapkan' }).click();
  await expect(page.getByText('2035 kcal', { exact: true })).toBeVisible();

  await page.getByRole('button', { name: 'AI Meal Planner' }).click();
  // Konfigurasi AI kini ada di Pengaturan (bukan inline di planner).
  await page.getByRole('button', { name: 'Pengaturan' }).click();
  await page.getByLabel('Model').fill('acceptance-model');
  await page.getByLabel('API Key').fill(mockedSecret);
  await page.getByRole('button', { name: 'Simpan Konfigurasi' }).click();
  await expect(page.getByText('Konfigurasi Tersimpan')).toBeVisible();
  await page.getByRole('button', { name: /Kembali/ }).click();
  await page.getByRole('button', { name: 'AI Meal Planner' }).click();
  // Asesmen PAGT: 4 langkah, cukup jawab "Tidak Ada" lalu buat plan.
  for (let step = 0; step < 4; step += 1) {
    await page.getByRole('button', { name: 'Tidak Ada' }).click();
  }
  await page.getByRole('button', { name: /Buat Plan Menu Sekarang/ }).click();
  await expect(page.getByText(/Rencana makan selesai/)).toBeVisible();
  await expect(page.getByText('Nasi Fixture').first()).toBeVisible();
  await page.getByRole('button', { name: 'Terapkan ke Dashboard' }).click();

  await page.getByRole('button', { name: 'Ekspor Laporan' }).click();
  await page.getByRole('button', { name: 'Ekspor PDF' }).click();
  await expect(page.getByText(/Laporan tersimpan/)).toBeVisible();
  const smokeState = await page.evaluate(() => (window as Window & { __NUTRISURVEY_SMOKE__?: { state: { calls: Array<{ command: string; payloadKeys: string[] }>; serializedCalls: string[]; logs: string[]; rawSecretSeenOnlyInMemory: boolean; projectBytes: number[]; importedBytes: number[] } } }).__NUTRISURVEY_SMOKE__?.state);
  expect(smokeState?.calls.map(call => call.command)).toEqual(expect.arrayContaining([
    'food_status',
    'food_search',
    'food_recommendations',
    'plugin:dialog|open',
    'import_csv_from_path',
    'calculate_tdee',
    'generate_ai_menu',
    'export_report',
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
