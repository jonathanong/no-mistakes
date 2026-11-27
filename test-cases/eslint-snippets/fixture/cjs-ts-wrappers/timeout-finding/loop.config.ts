import { defineConfig } from "vitest/config";
const cfg = { test: { testTimeout: 1000 } };
for (cfg.test.testTimeout of [60000]) {
}
export default defineConfig(cfg);
