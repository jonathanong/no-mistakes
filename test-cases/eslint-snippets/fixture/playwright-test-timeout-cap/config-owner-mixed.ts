import { defineConfig, test } from "@playwright/test";
import { projects, config, budget } from "./external-owner";
export default defineConfig({ projects, ...config });
test.extend({ sample: [async ({}, use) => await use("owned"), { timeout: budget }] });
test.setTimeout(budget);
test.describe.configure({ timeout: budget });
test("owned registration", async ({}, info) => { info.setTimeout(budget); });
