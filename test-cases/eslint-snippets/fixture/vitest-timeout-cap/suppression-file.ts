// no-mistakes-disable-file vitest-timeout-cap
import { defineConfig } from 'vitest/config';
export default defineConfig({ test: { testTimeout: 60000 } });
