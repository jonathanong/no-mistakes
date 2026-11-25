export async function inspect(page) {
  const sentinel = { async scrollIntoViewIfNeeded() {} };
  await sentinel.scrollIntoViewIfNeeded();
  await page.locator('[data-testid="details"]').first().scrollIntoViewIfNeeded();
  await page.getByTestId("load-more").getByText("Details").scrollIntoViewIfNeeded();
}
