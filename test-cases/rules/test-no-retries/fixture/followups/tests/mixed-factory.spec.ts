import { test as base } from "@playwright/test";

declare const unknownRunner: typeof base;

function choose(flag: boolean) {
  if (flag) return unknownRunner;
  return base.extend({});
}
const suite = choose(Boolean(process.env.CI));

suite.describe.serial("ordered", () => {});
suite.describe.configure({ retries: 2 });
