import { test as base } from "@playwright/test";

const test = base.extend({
  account: async ({}, use) => {
    await use("owner");
  },
});
const chained = base.extend({ first: async ({}, use) => { await use(1); } }).extend({
  second: async ({}, use) => {
    await use(2);
  },
});

test.skip("same-file extend skip", async () => {});
chained.fixme("chained extend fixme", async () => {});
base.extend({}).skip("inline extend skip", async () => {});
test("annotation skip", { annotation: { type: "skip" } }, async () => {});
test.describe.skip("extended describe skip", () => {});

const ordinary = {
  extend() {
    return { skip() {} };
  },
};
ordinary.extend().skip("ordinary object extend is not a test binding");
