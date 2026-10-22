import { expect as check } from "@playwright/test";
import * as playwright from "@playwright/test";

let pw = playwright;
pw.expect(locator).toBeVisible({ timeout: 20000 });
var checkAlias;
var checkAlias = check;
checkAlias(locator).toBeVisible({ timeout: 20000 });
let expect = check;
expect(locator).toBeVisible({ timeout: 20000 });
const alias = alias;
alias(locator).toBeVisible({ timeout: 20000 });
function nested(customAssertions) {
  const check = customAssertions.expect;
  check(locator).toBeVisible({ timeout: 20000 });
}
const { test: renamed } = playwright;
renamed(locator).toBeVisible({ timeout: 20000 });
const { expect: named } = playwright;
named(locator).toBeVisible({ timeout: 20000 });
const a = b;
const b = a;
a(locator).toBeVisible({ timeout: 20000 });

// Modifiers preserve the Playwright expect receiver and its timeout policy.
check(locator).not.toBeVisible({ timeout: 20000 });
