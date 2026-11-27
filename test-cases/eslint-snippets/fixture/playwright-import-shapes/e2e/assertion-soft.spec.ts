declare const loc: unknown;

const { expect: e } = require("@playwright/test");
e.soft(loc).toBeVisible({ timeout: 30000 });

const pw = require("@playwright/test");
pw.expect.soft(loc).toBeVisible({ timeout: 30000 });
pw.expect.poll(() => loc).toBeVisible({ timeout: 30000 });
