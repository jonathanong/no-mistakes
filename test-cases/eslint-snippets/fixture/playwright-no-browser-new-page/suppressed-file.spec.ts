// no-mistakes-disable-file playwright-no-browser-new-page -- dedicated browser.newPage contract spec
import { test } from "@playwright/test";

test("allowed by file suppression", async ({ browser }) => {
  await browser.newPage();
});
