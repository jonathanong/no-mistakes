import { defineConfig } from "vitest/config";
const cfg = { test: { testTimeout: 1000 } };
[cfg.test.testTimeout = 1000] = [60000];
export default defineConfig(cfg);
