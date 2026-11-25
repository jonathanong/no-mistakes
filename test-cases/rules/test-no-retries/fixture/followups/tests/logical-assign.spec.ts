import { test as base } from "@playwright/test";

let suite: typeof base | undefined;
suite ??= base;

suite.describe.serial("ordered", () => {});
suite.describe.configure({ retries: 2 });
