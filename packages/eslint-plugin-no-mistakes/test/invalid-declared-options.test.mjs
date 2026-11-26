import assert from "node:assert/strict";
import { it } from "vitest";
import { lint } from "./helpers.mjs";

const source = "export {}\n";

const cases = [
  [
    "test-no-shared-state",
    { allowBeforeAllAssignments: false },
    { allowBeforeAllAssignments: "no" },
  ],
  ["test-no-skips", { allow: ["skip"] }, { allow: "skip" }],
  ["test-no-wall-time-assertions", { allowedPaths: ["bench/**"] }, { allowedPaths: "bench/**" }],
  ["no-argv-main-guard", { message: "compare real paths" }, { message: 1 }],
  [
    "no-banned-import-outside-allowed-paths",
    { checkedPathPatterns: ["src/**"] },
    { checkedPathPatterns: "src/**" },
  ],
  ["no-consecutive-unbatched-reads", { readMethods: ["get"] }, { readMethods: "get" }],
  [
    "no-global-fetch-outside-helper",
    { checkedPathPatterns: ["src/**"] },
    { checkedPathPatterns: "src/**" },
  ],
  [
    "no-inline-noop-promise-catch",
    { checkedPathPatterns: ["src/**"] },
    { checkedPathPatterns: "src/**" },
  ],
  ["no-object-mutation", { module: false }, { module: "no" }],
];

for (const [rule, valid, invalid] of cases) {
  it(`${rule} rejects an invalid option and still runs with a valid one`, () => {
    const config = (option) => ({ [`no-mistakes/${rule}`]: ["error", option] });
    assert.ok(Array.isArray(lint(source, config(valid), "sample.ts")));
    assert.throws(
      () => lint(source, config(invalid), "sample.ts"),
      /schema|invalid|should be|must be|type|Value/i,
    );
  });
}
