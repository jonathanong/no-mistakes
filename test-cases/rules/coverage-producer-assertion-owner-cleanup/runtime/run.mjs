import { deepStrictEqual, strictEqual } from "node:assert/strict";
import { events } from "./stubs/logger.mjs";
import { register } from "../fixture/src/registry.mts";
import * as producers from "../fixture/src/producers.mts";

register(() => events.push("shutdown:first"), () => events.push("shutdown:second"));
const shutdown = ["shutdown:first", "shutdown:second"];
const main = ["coverage started", "row executed", "row executed", "captureException"];
const cases = [
  ["run", [], [...main, "assert", ...shutdown]],
  ["loggerWithOwner", [], ["coverage started", "assert"]],
  ["mapWithOwner", [], ["row executed", "assert"]],
  ["flatMapWithOwner", [], ["row executed", "assert"]],
  ["sentryWithOwner", [], ["captureException", "assert"]],
  ["cleanupWithOwner", [], ["assert", ...shutdown]],
  ["withoutAssertion", [], [...main, ...shutdown]],
  ["cleanupOnly", [], shutdown],
  ["loggerOnly", [], ["coverage started"]],
  ["mapOnly", [[{ execute() { events.push("row executed"); return []; } }]], ["row executed"]],
  ["flatMapOnly", [[{ execute() { events.push("row executed"); return []; } }]], ["row executed"]],
  ["sentryOnly", [], ["captureException"]],
  ["noOwner", [], []],
];
for (const [name, args, expected] of cases) {
  events.length = 0;
  const result = await producers[name](...args);
  await new Promise(setImmediate);
  deepStrictEqual(events, expected, name);
  if (name === "noOwner") strictEqual(result, 1);
}
console.log(JSON.stringify({ checked: cases.length }));
