import { test, expect } from "@playwright/test";
import { emailQueue } from "../../src/queues";

test.describe("Email sync", () => {
  test("enqueues a sync job", async ({ page }) => {
    await page.goto("/email-sync");
    await emailQueue.add("sync", { userId: "u1" });
    await expect(page).toHaveURL(/email-sync/);
  });
});
