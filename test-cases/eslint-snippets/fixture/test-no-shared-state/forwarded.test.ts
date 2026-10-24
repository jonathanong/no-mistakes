import { test, describe, beforeEach, afterEach } from "vitest";
import * as v from "vitest";
import pw from "@playwright/test";
const items = [];
test["call"](null, "static call", () => { items.push(1); }); // report
v.test[`apply`](v, ["static apply", () => { items.push(1); }]); // report
pw.apply(null, ["default runner", () => { items.push(1); }]); // report
const run = v.test;
function named() { items.push(1); } // report
run.apply(null, ["named", named]);
// A hook inside a forwarded suite must not clean state for a sibling test.
describe["apply"](null, ["suite", () => {
  beforeEach[`apply`](null, [() => { items.length = 0; }]);
  test.apply(null, ["cleaned", () => { items.push(1); }]);
}]);
test["call"](null, "outside suite", () => { items.push(1); }); // report
const maps = new Map();
describe[`call`](null, "named cleanup suite", () => {
  function reset() { maps.clear(); }
  beforeEach["call"](null, reset);
  test["apply"](null, ["cleaned named", () => { maps.set("x", 1); }]);
});
describe.call(null, "after cleanup suite", () => {
  afterEach["apply"](null, [() => { maps.clear(); }]);
  test[`call`](null, "cleaned after", () => { maps.set("x", 1); });
});
const plain = new Map();
function mutate() { plain.set("x", 1); } // report
v.test["apply"](v, ["named map", mutate]);
const outside = new Map();
v.describe[`apply`](v, ["namespace suite", () => {
  v.beforeEach["apply"](v, [() => { outside.clear(); }]);
  v.test["apply"](v, ["cleaned namespace", () => { outside.set("x", 1); }]);
}]);
v.test["apply"](v, ["outside namespace suite", () => { outside.set("x", 1); }]); // report
// Type wrappers around the method, literal list, and callback keep the registration.
(test.apply as typeof test.apply)(null, ["asserted list", () => { items.push(1); }] as const); // report
test.apply(null, ["asserted callback", (() => { items.push(1); }) as () => void]); // report
