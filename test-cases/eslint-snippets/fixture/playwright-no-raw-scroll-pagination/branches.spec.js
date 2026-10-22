import { test } from "@playwright/test";

const scrollTo = window?.scrollTo;
scrollTo(0, 100);
const jump = (window?.document).scrollTo;
const scrollBy = window?.document.scrollTo;
scrollBy(0, 100);
(window?.document).scrollTo(0, 100);
var scroll;
var scroll = window.scroll;
scroll(0, 100);
const wait = page.waitForResponse;
await wait((req) => req.url().includes("?cursor=1"));
page.url().searchParams?.has("after");
page.url().searchParams.has.call(page.url().searchParams, "after");
void test;
