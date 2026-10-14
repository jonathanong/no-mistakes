import { defineConfig } from "vitest/config";
import { it, beforeEach, vi } from "vitest";
import { projects, budget, options } from "./external-owner";
export default defineConfig({ test: { projects, testTimeout: budget, hookTimeout: 5000 } });
it("unknown trailing budget", async () => {}, budget);
beforeEach(async () => {}, budget);
vi.setConfig({ testTimeout: budget });
it("unknown options", options, async () => {});
it("known oversized override", async () => {}, 30001);
