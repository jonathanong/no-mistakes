export async function inspectWithCursor(page) {
  await page.waitForRequest("**/api/list?after=example");
  await page.evaluate(() => window.scrollTo(0, 0));
}
