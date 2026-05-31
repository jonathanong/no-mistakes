import { test } from "@playwright/test";

test("invalid", async ({ page }) => {
  await page.goto("/broken")
