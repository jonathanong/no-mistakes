import pw = require("@playwright/test");

function pick(page: { locator(selector: string): unknown }) {
  return page.locator(".btn");
}

void pw;
void pick;
