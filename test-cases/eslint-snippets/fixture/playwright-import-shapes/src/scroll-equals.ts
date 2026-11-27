import pw = require("@playwright/test");

async function scroll(page: { waitForResponse(url: string): Promise<unknown> }) {
  await page.waitForResponse("?cursor=1");
  window.scrollTo(0, 100);
}

void pw;
