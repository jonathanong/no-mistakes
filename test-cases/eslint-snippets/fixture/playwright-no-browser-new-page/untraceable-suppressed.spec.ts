import { test } from "@example/e2e-fixtures";

// no-mistakes-disable-next-line playwright-no-browser-new-page -- verifies raw browser.newPage behavior
test("loads orders", async ({ browser }) => {
  await browser.newPage();
});

test("loads orders again", async ({ browser }) => {
  await browser.newPage();
});
