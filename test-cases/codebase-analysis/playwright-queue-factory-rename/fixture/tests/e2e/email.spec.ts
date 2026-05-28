import { test, expect } from "@playwright/test";
import { Queue } from "../../src/queue-impl";

test.describe("Email factory", () => {
  test("constructs the email queue", async ({ page }) => {
    // Reference the queue by its name literally — the diff-aware queue
    // hint indexes `new Queue('emails')` on the dependent side. The Queue
    // class import points at queue-impl.ts so the BFS doesn't reach this
    // spec through a graph dependency on the diff-modified queues.ts.
    const local = new Queue("emails");
    await page.goto("/welcome");
    await expect(page).toHaveURL(/welcome/);
    await local.add("greet", { userId: "u1" });
  });
});
