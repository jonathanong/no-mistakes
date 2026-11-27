declare const loc: unknown;

const { expect } = require("./fixtures");
expect(loc).toBeVisible({ timeout: 30000 });
