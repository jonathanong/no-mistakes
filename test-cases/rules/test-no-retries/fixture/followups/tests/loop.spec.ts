import { test as base } from "@playwright/test";

for (const suite of [base]) {
  suite.describe.serial("ordered", () => {});
  suite.describe.configure({ retries: 2 });
}
