import { test } from "@playwright/test";

function demo(page: { locator: (selector: string) => void }) {
  // `using` is not a single-write const, let, or var alias.
  using locator = page.locator;
  locator(".a");
}
void test;
void demo;
