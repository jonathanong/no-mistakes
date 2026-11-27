import test = require("@playwright/test");

test("loads orders", async ({ browser }) => {
  await browser.newPage();
});
