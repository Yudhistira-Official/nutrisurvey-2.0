import { defineConfig } from 'eslint/config';
import nextConfig from 'eslint-config-next';

export default defineConfig([
  {
    ignores: ['.next/**', 'out/**', 'src-tauri/target/**', 'node_modules/**'],
  },
  ...nextConfig,
  {
    rules: {
      'react-hooks/set-state-in-effect': 'off',
    },
  },
]);
