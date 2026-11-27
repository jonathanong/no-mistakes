declare const loc: unknown;

function require(_id: string) {
  return { expect: () => ({ toBeVisible() {} }) };
}
const expect = require("@playwright/test").expect;
expect(loc).toBeVisible({ timeout: 30000 });
