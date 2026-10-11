import { defineConfig } from 'vitest/config';
export default defineConfig({ test: { testTimeout: 30001, hookTimeout: 30001, projects: [
  { extends: './base.config.ts', test: { name: 'base', hookTimeout: 0 } },
  { extends: './missing.config.ts', test: { name: 'missing', hookTimeout: 1 } },
] } });
