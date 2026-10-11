import { defineConfig } from 'vitest/config';
export default defineConfig({ test: { name: 'negative', testTimeout: -1, hookTimeout: 30000 } });
