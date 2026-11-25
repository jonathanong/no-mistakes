const pw = require("@playwright/test");
const { test } = require("@playwright/test");

pw.test("cjs namespace", async ({ browser }) => {
  await browser.newPage();
});

test("cjs destructured", async ({ browser }) => {
  await browser.newPage();
});
