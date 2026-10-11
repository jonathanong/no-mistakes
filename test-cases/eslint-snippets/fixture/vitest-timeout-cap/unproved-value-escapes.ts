import { test } from "vitest";
const yielded = { timeout: 1 };
function* expose() { yield yielded; }
test("yield escape", yielded, () => {});
const awaited = { timeout: 1 };
async function wait() { await awaited; }
test("await escape", awaited, () => {});
const tagged = { timeout: 1 };
tag`${tagged}`;
test("tagged escape", tagged, () => {});
const keyed = { timeout: 1 };
const carrier = { [keyed]: 1 };
test("computed key escape", keyed, () => {});
