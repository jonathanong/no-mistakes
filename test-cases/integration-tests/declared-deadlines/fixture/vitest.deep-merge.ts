import { mergeConfig as merge, defineConfig } from 'vitest/config';
const config = merge(defineConfig({ test: { testTimeout: 30001, hookTimeout: 30000 } }),
  { test: { hookTimeout: 1 } });
export default config;
