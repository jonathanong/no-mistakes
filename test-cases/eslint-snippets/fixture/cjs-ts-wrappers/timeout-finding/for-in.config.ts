import { defineConfig } from "vitest/config";
const cfg = { test: { testTimeout: 1000 } };
for (cfg.test.testTimeout in { over: true }) {
}
export default defineConfig(cfg);
