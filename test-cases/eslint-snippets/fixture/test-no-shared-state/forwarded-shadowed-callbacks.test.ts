import { test } from "vitest";
import * as v from "vitest";
const items = [];
test("outer", () => {
  // An active outer test must not make a shadowed runner's callback inline.
  function run(test) {
    test.apply(null, ["unused", () => { items.push(1); }]);
  }
  run(() => {});
  ((v) => {
    v.test.call(null, "unused namespace", () => { items.push(1); });
  })({ test() {} });
});
