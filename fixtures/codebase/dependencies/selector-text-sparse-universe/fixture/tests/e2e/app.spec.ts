import { test } from "@playwright/test";

test("covers visible component text", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Discuss" }).click();
});
