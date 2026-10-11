import { test } from "vitest";
const extended = test.extend({ owned: [async ({}, use) => use(1), { auto: true }] });
extended("extended oversized", { timeout: 30001 }, () => {});
extended.beforeEach(() => {}, 0);
const overridden = extended.override("owned", { scope: "test" }, async () => 1);
overridden("overridden oversized", () => {}, 30001);
const scoped = extended.scoped({ owned: [async ({}, use) => use(1), { auto: true }] });
scoped("scoped invalid", { timeout: -1 }, () => {});
const worker = test.extend("owned", { scope: "worker" }, async () => 1);
worker("worker registrar positive", { timeout: 30000 }, () => {});
const file = test.extend("owned", { scope: "file" }, async () => 1);
file("file registrar positive", { timeout: 1 }, () => {});
// A fixture value's timeout property is data, not a FixtureOptions deadline.
const payload = test.extend("payload", { timeout: 30001 });
payload("fixture data positive", { timeout: 1 }, () => {});
