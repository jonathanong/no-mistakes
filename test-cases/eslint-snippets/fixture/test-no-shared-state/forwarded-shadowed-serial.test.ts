import { test, describe, beforeAll } from "@playwright/test";
import * as v from "@playwright/test";
const registry = new Map();
// A shadow's configure call cannot turn the actual runner serial.
function register(describe, v) {
  describe.configure.call(describe, { mode: "serial" });
  v.describe.configure.apply(v.describe, [{ mode: "serial" }]);
}
beforeAll.apply(null, [() => { registry.clear(); }]);
test.apply(null, ["still concurrent", () => { registry.set("x", 1); }]); // report
