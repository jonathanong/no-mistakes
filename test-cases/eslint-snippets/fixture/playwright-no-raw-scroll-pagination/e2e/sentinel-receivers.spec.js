export async function scrollToSentinels(page) {
  await page.getByTestId("load-more").first().scrollIntoViewIfNeeded();
  await page.locator('[data-testid="load-more"]').scrollIntoViewIfNeeded();
  await page.locator("[data-testid='load-more']").nth(0).scrollIntoViewIfNeeded();
  await page.locator("[data-testid=load-more]").scrollIntoViewIfNeeded();
  const sentinel = page.locator(".load-more");
  await sentinel.last().scrollIntoViewIfNeeded();
  const alias = page.getByTestId("load-more");
  await alias.filter({ visible: true }).scrollIntoViewIfNeeded();
}
