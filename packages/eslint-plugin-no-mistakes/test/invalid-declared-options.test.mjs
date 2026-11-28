import assert from "node:assert/strict";
import { it } from "vitest";
import { lint } from "./helpers.mjs";

const source = "export {}\n";

const cases = [
  [
    "test-no-shared-state",
    "allowBeforeAllAssignments",
    { allowBeforeAllAssignments: false },
    { allowBeforeAllAssignments: "no" },
  ],
  ["test-no-skips", "allow", { allow: ["skip"] }, { allow: "skip" }],
  [
    "test-no-config-mirror-assertion",
    "fixtureDirs",
    { fixtureDirs: ["fixtures"], parsers: [{ module: "smol-toml", name: "parse" }] },
    { fixtureDirs: "fixtures" },
  ],
  [
    "test-no-wall-time-assertions",
    "allowedPaths",
    { allowedPaths: ["bench/**"] },
    { allowedPaths: "bench/**" },
  ],
  ["no-argv-main-guard", "message", { message: "compare real paths" }, { message: 1 }],
  [
    "no-banned-import-outside-allowed-paths",
    "checkedPathPatterns",
    { checkedPathPatterns: ["src/**"] },
    { checkedPathPatterns: "src/**" },
  ],
  [
    "no-consecutive-unbatched-reads",
    "readMethods",
    { readMethods: ["get"] },
    { readMethods: "get" },
  ],
  [
    "no-global-fetch-outside-helper",
    "checkedPathPatterns",
    { checkedPathPatterns: ["src/**"] },
    { checkedPathPatterns: "src/**" },
  ],
  [
    "no-inline-noop-promise-catch",
    "checkedPathPatterns",
    { checkedPathPatterns: ["src/**"] },
    { checkedPathPatterns: "src/**" },
  ],
  ["no-object-mutation", "module", { module: false }, { module: "no" }],
];

function rejectedValue(value) {
  const shown = typeof value === "string" ? `"${value}"` : String(value);
  return shown.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

for (const [rule, key, valid, invalid] of cases) {
  it(`${rule} rejects an invalid option and still runs with a valid one`, () => {
    const config = (option) => ({ [`no-mistakes/${rule}`]: ["error", option] });
    assert.deepEqual(lint(source, config(valid), "sample.ts"), []);
    assert.deepEqual(Object.keys(invalid), [key]);
    assert.throws(
      () => lint(source, config(invalid), "sample.ts"),
      (error) => {
        assert.match(error.message, new RegExp(`Key "no-mistakes/${rule}":`));
        assert.match(error.message, new RegExp(`Value ${rejectedValue(invalid[key])} should be`));
        return true;
      },
    );
  });
}
