import { test as base } from "@playwright/test";

class Suite {
  runner = base.extend({});
  unrelated = makeUnrelatedRunner();
}

new Suite().unrelated.describe.serial("ordered", () => {});
new Suite().unrelated.describe.configure({ retries: 2 });
