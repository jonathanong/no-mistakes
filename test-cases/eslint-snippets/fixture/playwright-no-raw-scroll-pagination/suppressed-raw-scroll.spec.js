export async function inspectWithCursor(page) {
  await page.waitForRequest("**/api/list?after=example");
  // no-mistakes-disable-next-line playwright-no-raw-scroll-pagination -- deliberately inspect this scroll
  await page.evaluate(() => window.scrollTo(0, 0));
}
