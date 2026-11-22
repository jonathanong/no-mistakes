import { configDefaults as defaults, defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    exclude: ['**/generated/**'],
    projects: [
      { test: { name: 'unit', exclude: ['**/node_modules/**'] } },
      { test: { name: 'integration', exclude: [...defaults.exclude, '**/generated/**'] } },
    ],
  },
});
