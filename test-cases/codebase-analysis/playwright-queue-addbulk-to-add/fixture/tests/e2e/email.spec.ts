import { test, expect } from "@playwright/test";
import { emailQueue } from "../../src/queues";

test.describe("Email greet", () => {
  test("enqueues a greet job", async ({ page }) => {
    await page.goto("/welcome");
    await emailQueue.add("greet", { userId: "u1" });
    await expect(page).toHaveURL(/welcome/);
  });
});
