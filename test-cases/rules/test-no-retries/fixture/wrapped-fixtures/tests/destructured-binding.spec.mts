import { test as base } from "@playwright/test";

const { test: t } = { test: base };
t.describe.serial("ordered", () => {});
t.describe.configure({ retries: 2 });
