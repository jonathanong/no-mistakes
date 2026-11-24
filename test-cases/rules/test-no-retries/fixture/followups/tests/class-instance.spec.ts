import { test as base } from "@playwright/test";

const Suite = class {
  static other = base;
  runner = base.extend({});
};

new Suite().runner.describe.serial("ordered", () => {});
new Suite()["runner"].describe.configure({ retries: 2 });
