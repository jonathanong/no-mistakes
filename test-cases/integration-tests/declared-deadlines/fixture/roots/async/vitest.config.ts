import { defineConfig } from 'vitest/config';
export default defineConfig(async () => ({ test: { hookTimeout: 1000 } }));
