import { defineConfig, test } from "@playwright/test";
import { method, config } from "./external-owner";
export default defineConfig(config, config);
defineConfig({ timeout: 30001, projects: [{ timeout: 0 }, { timeout: 1 / 0 }] });
test.extend({ sample: [async ({}, use) => await use("owned"), { timeout: 0 }] });
test.extend({ sample: [async ({}, use) => await use("owned"), { timeout: Infinity }] });
test[method]("unknown API", async () => {});
defineConfig(config, config);
