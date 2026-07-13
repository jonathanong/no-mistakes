import { test } from "@playwright/test";

// Keep a second file to exercise shared app-text indexing across parallel test analysis.
test("also covers visible component text", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Discuss" }).click();
});
