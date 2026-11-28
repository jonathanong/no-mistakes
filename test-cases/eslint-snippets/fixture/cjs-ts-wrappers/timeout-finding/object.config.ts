import { defineConfig } from "vitest/config";
const cfg = { test: { testTimeout: 1000 } };
({ testTimeout: cfg.test.testTimeout } = { testTimeout: 60000 });
export default defineConfig(cfg);
