import { defineConfig } from 'vitest/config';
declare const opaque: object;
export default defineConfig({ ['test']: { testTimeout: 30000, hookTimeout: 1, projects: [
  { test: { name: 'literal', ['testTimeout']: 1, ['hookTimeout']: 30000 } },
  { test: { name: 'invalid-literal', ['testTimeout']: 0, ['hookTimeout']: 30001 } },
  { test: { name: 'getter', get testTimeout() { return 30001; } } },
  { test: { name: 'unrelated-getter', get unrelated() { return 30001; } } },
  { extends: chooseBase(), test: { name: 'dynamic-extends', testTimeout: 1 } },
  { extends: false, ...opaque, test: { name: 'opaque-extends', testTimeout: 1 } },
  { ...opaque, extends: false, test: { name: 'false-restored', testTimeout: 1 } },
  { test: { name: 'ignored-test-extends', extends: false } },
] } });
