import { test as base } from "@playwright/test";

let t = base.extend({});
t.describe.serial("ordered", () => {});
t.describe.configure({ retries: 2 });
