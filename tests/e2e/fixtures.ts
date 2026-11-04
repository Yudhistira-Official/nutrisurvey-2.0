import { chromium, expect, test as base } from '@playwright/test';
import type { Browser } from '@playwright/test';

// NUTRISURVEY_CDP_ENDPOINT: konek ke browser CDP eksternal (Lightpanda, dsb.)
// alih-alih meluncurkan Chromium bawaan Playwright. Tanpa env, berperilaku
// normal (Chromium bawaan).
const cdpEndpoint = process.env.NUTRISURVEY_CDP_ENDPOINT;

export const test = base.extend<{ browser: Browser }>({
  browser: async ({ }, useFixture) => {
    const browser = cdpEndpoint
      ? await chromium.connectOverCDP(cdpEndpoint, { timeout: 60_000 })
      : await chromium.launch();
    await useFixture(browser);
    await browser.close();
  },
});

export { expect };
