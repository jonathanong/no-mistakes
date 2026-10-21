import { test } from "@playwright/test";

await page["waitForResponse"]("**/api?cursor=1");
window["scrollTo"](0, 100);
window[`scrollBy`](0, 100);
(window?.scroll)(0, 100);
const { scrollTo } = window;
scrollTo(0, 100);
window.scrollTo.call(window, 0, 100);
const go = scrollBy;
go(0, 100);
void test;
