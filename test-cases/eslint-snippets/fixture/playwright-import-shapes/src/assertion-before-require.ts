declare const loc: unknown;

const run = () => check(loc).toBeVisible({ timeout: 30000 });
const { expect: check } = require("@playwright/test");
run();
