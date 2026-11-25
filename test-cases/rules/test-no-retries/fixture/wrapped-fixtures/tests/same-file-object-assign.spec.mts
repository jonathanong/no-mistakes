import { test as base } from "@playwright/test";

function wrap(t: typeof base) {
  return Object.assign({}, t);
}

const t = wrap(base);
t.describe.serial("ordered", () => {});
t.describe.configure({ retries: 2 });
