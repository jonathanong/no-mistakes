import { defineConfig } from 'vitest/config';
const low = { test: { testTimeout: 1 } };
export default defineConfig({ test: { testTimeout: 30000, hookTimeout: 30000 }, ...low });
