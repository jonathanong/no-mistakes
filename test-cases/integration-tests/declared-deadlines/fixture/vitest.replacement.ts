import { defineConfig } from 'vitest/config';
export default defineConfig({ test: { testTimeout: 0, hookTimeout: 30001 },
  ...{ test: { testTimeout: getBudget() } },
});
