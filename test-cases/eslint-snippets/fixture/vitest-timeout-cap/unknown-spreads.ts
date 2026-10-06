import { defineConfig, mergeConfig } from 'vitest/config';
const base = { test: { testTimeout: 60000, hookTimeout: 60000, projects: [{ test: { testTimeout: 60000 } }] } };
const uncertain = { ...dynamicConfig };
const restored = { ...uncertain, test: { testTimeout: 5000 } };
export default defineConfig(mergeConfig(base, restored));
