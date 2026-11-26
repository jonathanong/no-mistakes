const pw = require("@playwright/test");
const { test } = require("@playwright/test");
const directTest = require("@playwright/test").test;

pw.test("cjs namespace", async ({ browser }) => {
  await browser.newPage();
});

test("cjs destructured", async ({ browser }) => {
  await browser.newPage();
});

require("@playwright/test").test("cjs member call", async ({ browser }) => {
  await browser.newPage();
});

directTest("cjs member alias", async ({ browser }) => {
  await browser.newPage();
});
