import { expect as check } from "@playwright/test";
import * as playwright from "@playwright/test";

let verify = check;
verify(locator).toBeVisible({ timeout: 20000 });
let pw = playwright;
pw.expect(locator).toBeVisible({ timeout: 20000 });
let { expect: named } = playwright;
named(locator).toHaveText("text", { timeout: 20000 });
const { expect: fallbackNamed = check } = playwright;
fallbackNamed(locator).toBeVisible({ timeout: 20000 });

// Reassignment fails closed.
let rebound = check;
rebound = custom;
rebound(locator).toBeVisible({ timeout: 20000 });
let loose = playwright;
loose = other;
loose.expect(locator).toBeVisible({ timeout: 20000 });
