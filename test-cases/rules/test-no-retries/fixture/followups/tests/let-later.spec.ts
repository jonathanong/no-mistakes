import { test as base } from "@playwright/test";

let suite: typeof base;
suite = base.extend({});
suite ??= base;

suite.describe.serial("ordered", () => {});
suite.describe.configure({ retries: 2 });
