import { defineConfig } from 'vitest/config';

export default defineConfig({ test: { testTimeout: 30000, hookTimeout: 1, projects: [
  { test: { name: 'hook-getter', get hookTimeout() { return 30001; } } },
  { name: 'test-getter', get test() { return { testTimeout: 30001 }; } },
  { get extends() { return false; }, test: { name: 'extends-getter', testTimeout: 1 } },
] } });
