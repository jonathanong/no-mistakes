await expect(locator).toBeVisible({ timeout: 1000 });
await page.getByRole("button", { name: "Save" }).click();
await page.locator("[data-pw=save]").click();
