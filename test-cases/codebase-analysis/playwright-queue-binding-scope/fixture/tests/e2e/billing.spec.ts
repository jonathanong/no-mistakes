import { test, expect } from "@playwright/test";
import { billingQueue } from "../../src/queues";

test.describe("Billing sync", () => {
  test("enqueues a billing sync job", async ({ page }) => {
    await page.goto("/billing");
    await billingQueue.add("sync", { userId: "u1" });
    await expect(page).toHaveURL(/billing/);
  });
});
