declare const loc: unknown;

const expect = require("@playwright/test").expect;
expect(loc).toBeVisible({ timeout: 30000 });
