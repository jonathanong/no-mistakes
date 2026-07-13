import { test } from "@playwright/test";

test("finds visible text after navigation", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Discuss" }).click();
});
