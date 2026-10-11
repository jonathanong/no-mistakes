import { defineConfig } from 'vitest/config';
import { budgets } from './budgets';
declare const opaque: object;
export default defineConfig({ test: {
  ...budgets,
  projects: [
    { test: { name: 'inherited', extends: true } },
    { test: { name: 'invalid', testTimeout: 0, hookTimeout: 30001 } },
    { test: { name: 'negative', testTimeout: -1 } },
    { extends: false, test: { name: 'absent' } },
    { test: { name: 'opaque-last', testTimeout: 1, hookTimeout: 30000, ...opaque } },
    { test: { name: 'restored', ...opaque, testTimeout: 30000, hookTimeout: 1 } },
    { test: { name: 'unknown-expression', testTimeout: getBudget(), hookTimeout: 1e400 } },
  ],
} });
