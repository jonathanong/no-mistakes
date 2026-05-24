const routes = {
  link: () => 'a[href="/helper-click"]',
};

await page.click();
await page.click(link());
