import { test } from "@playwright/test";

let wait = page.waitForResponse;
await wait((response) => response.url().includes("?cursor=1"));
let scrollTo = window.scrollTo;
scrollTo(0, 100);
var jump = window.scrollBy;
jump(0, 50);

// Reassignment fails closed, so this scroll is not the window global.
let rebound = window.scrollTo;
rebound = localScroll;
rebound(0, 10);

void test;
