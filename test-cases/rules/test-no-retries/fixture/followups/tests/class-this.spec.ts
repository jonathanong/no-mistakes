import { test as base } from "@playwright/test";

class Suite {
  runner = base.extend({});

  define() {
    this.runner.describe.serial("ordered", () => {});
    this.runner.describe.configure({ retries: 2 });
  }
}
