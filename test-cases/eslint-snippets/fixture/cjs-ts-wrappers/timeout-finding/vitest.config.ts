import { defineConfig } from "vitest/config";
const cfg = { test: { testTimeout: 1000 } };
console.log(cfg.test.testTimeout as number);
export default defineConfig(cfg);
