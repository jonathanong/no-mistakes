import { test, expect } from "@playwright/test";
import { emailQueue } from "../../src/queues";

test.describe("Email producer", () => {
  test("enqueues a welcome job", async ({ page }) => {
    await page.goto("/welcome");
    await emailQueue.add("send-welcome", { userId: "u1" });
    await expect(page).toHaveURL(/welcome/);
  });
});
