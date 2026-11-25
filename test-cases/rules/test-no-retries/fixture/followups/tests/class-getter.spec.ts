import { test as base } from "@playwright/test";

new (class {
  get suite() {
    return base;
  }
  other() {
    return base;
  }
})().suite.describe.configure({ retries: 2 });

class UnrelatedMethodSuite {
  other() {
    return base;
  }
  plain = { describe: { serial: (_name: string, _fn: () => void) => {} } };
}

new UnrelatedMethodSuite().plain.describe.serial("unrelated method", () => {});
