const routes = {
  link: () => 'a[href="/helper-click"]',
};
const { link } = routes;

await page.click();
await page.click(link());
