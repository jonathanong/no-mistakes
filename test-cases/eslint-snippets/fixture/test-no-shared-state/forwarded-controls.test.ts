import { test } from "vitest";
import * as v from "vitest";
import vitest from "vitest";
const items = [];
declare const args: any[];
declare const key: string;
// Unknown argument lists cannot establish a registered callback.
test.apply(null, args);
test.apply(null, ["spread", ...args, () => { items.push(1); }]);
test.apply(null, [, () => { items.push(1); }]);
test.call(null, ...args, () => { items.push(1); });
test[key](null, "dynamic", () => { items.push(1); });
function shadow(test, v) {
  test["apply"](null, ["shadowed", () => { items.push(1); }]);
  v.test["call"](null, "shadowed namespace", () => { items.push(1); });
}
vitest.apply(null, ["not a runner", () => { items.push(1); }]);
test.apply(null, ["unregistered closure", () => {
  const unrelated = [() => { items.push(1); }];
}]);
// A receiver callback is not the registered callback.
test.call(() => { items.push(1); }, "receiver", () => {});
// Nested intrinsic forwarding invokes its explicit receiver, not the spelling test.
declare const other: (...args: any[]) => unknown;
test.call.call(other, null, "another function", () => { items.push(1); });
test.apply.call(other, null, ["another function", () => { items.push(1); }]);
test.call.apply(other, [null, "another function", () => { items.push(1); }]);
// Arbitrary nested arrays are not callback registrations, even inside an apply list.
test.apply(null, ["nested container", [() => { items.push(1); }]]);
test.apply.apply(test, [null, ["nested unknown", ...args, () => { items.push(1); }]]);
test.apply.apply(test, [null, ["nested container", [() => { items.push(1); }]]]);
