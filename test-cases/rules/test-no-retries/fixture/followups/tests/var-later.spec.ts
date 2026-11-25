import { test as base } from "@playwright/test";

var suite: typeof base;
suite = base.extend({});

suite.describe.serial("ordered", () => {});
suite.describe.configure({ retries: 2 });
