import { test } from "@playwright/test";

test("moves around the list", async ({ page }) => {
  const heading = page.getByTestId("heading");
  await heading.scrollIntoViewIfNeeded();
  await page.getByTestId("heading").scrollIntoViewIfNeeded();
  await page.getByTestId(dynamicId).scrollIntoViewIfNeeded();
  await page.getByTestId(`load-${suffix}`).scrollIntoViewIfNeeded();
  await page.getByRole("heading").scrollIntoViewIfNeeded();
  await locators.heading.scrollIntoViewIfNeeded();
  await page.mouse.wheel(0, 200);
});
