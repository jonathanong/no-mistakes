export async function inspectWithCursor(page) {
  await page.waitForRequest("**/api/list?after=example");
  await page.evaluate(() => window.scrollTo(0, 0)); // no-mistakes-disable-line playwright-no-raw-scroll-pagination -- kept for this fixture
}
