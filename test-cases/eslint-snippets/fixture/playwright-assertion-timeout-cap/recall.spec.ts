import { expect as check } from "@playwright/test";
import * as pw from "@playwright/test";

check(locator).toBeVisible({ timeout: 20000 });
pw.expect(locator)["toBeVisible"]({ [`timeout`]: 20000 });
(pw?.expect)(locator).toBeVisible({ timeout: 20000 });
check(locator).toBeVisible.call(check(locator), { timeout: 20000 });
function helper(check) {
  check(locator).toBeVisible({ timeout: 20000 });
}
const customAssertions = { expect };
customAssertions.expect(locator).toHaveText("text", { timeout: 30000 });
const checkAlias = customAssertions.expect;
checkAlias(locator).toHaveText("text", { timeout: 30000 });
