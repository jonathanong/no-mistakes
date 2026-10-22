import * as pw from "@playwright/test";
// Rest entries do not hide the actual expect property or crash alias resolution.
const { expect: check, ...rest } = pw;
check(locator).toBeVisible({ timeout: 20000 });
rest.toBeVisible({ timeout: 20000 });
