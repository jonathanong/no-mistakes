// no-mistakes-disable-file playwright-no-raw-scroll-pagination -- kept for this fixture
export async function inspectWithCursor(page) {
  await page.waitForRequest("**/api/list?after=example");
  await page.evaluate(() => window.scrollTo(0, 0));
}
