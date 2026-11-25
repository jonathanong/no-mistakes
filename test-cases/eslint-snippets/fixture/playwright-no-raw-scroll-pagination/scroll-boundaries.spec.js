export async function inspect(page, args) {
  await page.waitForRequest.apply(page, args);
  await page.waitForRequest((req) => req.url().has("after"));
  await page.evaluate(() => window.scrollTo(0, 0));
}
