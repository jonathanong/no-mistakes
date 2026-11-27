declare const page: { locator(selector: string): unknown };

function check(page: { locator(selector: string): unknown }) {
  page.locator(".item");
}

const { test } = require("@playwright/test");
void test;
void check;
