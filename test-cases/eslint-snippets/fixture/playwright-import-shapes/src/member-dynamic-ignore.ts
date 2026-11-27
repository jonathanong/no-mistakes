declare const exportName: string;
declare const loc: unknown;

const check = require("@playwright/test")[exportName];
check(loc).toBeVisible({ timeout: 30000 });
