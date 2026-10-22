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
const { expect: named } = playwright;
named(locator).toBeVisible({ timeout: 20000 });
const a = b;
const b = a;
a(locator).toBeVisible({ timeout: 20000 });
