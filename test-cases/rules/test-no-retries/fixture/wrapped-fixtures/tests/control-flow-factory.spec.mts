import { test as base } from "@playwright/test";

function makeTest(kind: string) {
  switch (kind) {
    case "a":
      return base.extend({});
    default:
      return base;
  }
}

const t = makeTest("a");
t.describe.serial("ordered", () => {});
t.describe.configure({ retries: 2 });
