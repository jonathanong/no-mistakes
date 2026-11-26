import { test } from "./fixtures";

test("uses configured fixture names", async ({ customBrowser }) => {
  await customBrowser.newPage();
});

test("uses default browser name", async ({ browser }) => {
  await browser.newPage();
});
