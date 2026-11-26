import { test } from "@playwright/test";

test("suppressions", async ({ browser }) => {
  // no-mistakes-disable-next-line playwright-no-browser-new-page -- intentional secondary page test
  await browser.newPage();
  await browser.newPage(); // no-mistakes-disable-line no-mistakes/playwright-no-browser-new-page
  await browser.newPage();
});
