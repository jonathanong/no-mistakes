import * as v from "@playwright/test";
const registry = new Map();
v.describe.configure.apply(v.describe, [{ mode: "serial" }]);
v.test.beforeAll.apply(v.test, [() => { registry.clear(); }]);
v.test.apply(v, ["serial cleanup", () => { registry.set("x", 1); }]);
