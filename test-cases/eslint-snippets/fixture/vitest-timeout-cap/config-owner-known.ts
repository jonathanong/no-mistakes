import { defineConfig } from "vitest/config";
import { it, vi } from "vitest";
import { method, budget } from "./external-owner";
export default defineConfig({ test: { testTimeout: 30001, hookTimeout: 0,
  projects: [{ test: { testTimeout: -1, hookTimeout: 1 / 0 } }] } });
vi[method]({ testTimeout: budget });
it("known legal", async () => {}, /* no-mistakes-timeout-reason: isolated worker startup needs this budget */ 30000);
it("unresolved nonfinite override", async () => {}, Infinity);
