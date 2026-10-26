import { test } from "@playwright/test";

await page.waitForResponse(() => url.searchParams.has.call(url.searchParams, "cursor"));
await page.waitForRequest(() => url.searchParams.get.apply(url.searchParams, ["after"]));
window.scrollTo(0, 100);
void test;
