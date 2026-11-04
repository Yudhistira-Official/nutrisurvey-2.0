import { defineConfig } from 'eslint/config';
import nextConfig from 'eslint-config-next';

export default defineConfig([
  {
    ignores: ['.next/**', 'out/**', 'src-tauri/target/**', 'node_modules/**', '.worktrees/**', '**/target/**'],
  },
  ...nextConfig,
  {
    rules: {
      'react-hooks/set-state-in-effect': 'off',
    },
  },
  {
    // Fixture Playwright memakai parameter bernama `useFixture`/callback fixture
    // yang bukan React Hook — matikan aturan hooks khusus berkas e2e.
    files: ['tests/e2e/**/*.ts'],
    rules: {
      'react-hooks/rules-of-hooks': 'off',
    },
  },
]);
