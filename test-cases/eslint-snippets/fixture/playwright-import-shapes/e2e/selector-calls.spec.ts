import { test } from "@playwright/test";
declare const page: { locator(selector: string): unknown };

(0, page.locator)(".btn");
page.locator.call(page, ".btn");
page.locator.call(page, ...[".btn"]);
page.locator.apply(page, [".btn"]);
void test;
