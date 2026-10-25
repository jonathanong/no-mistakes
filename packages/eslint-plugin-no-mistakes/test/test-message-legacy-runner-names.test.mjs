import { describe, expect, it } from "vitest";
import { fixture, lint } from "./helpers.mjs";

const rule = "test-no-error-message-matching";
function check(name) {
  return lint(fixture(`${rule}/${name}`), { [`no-mistakes/${rule}`]: "error" }, name);
}
describe("legacy named test runner reporting", () => {
  it("keeps test/it checks without assuming renamed parameters are runners", () => {
    expect(
      check("legacy-runner-names.test.ts").map(({ line, messageId }) => [line, messageId]),
    ).toEqual([3, 7, 10, 19, 23].map((line) => [line, "message"]));
  });
  it("preserves CommonJS module runner aliases", () => {
    const findings = lint(
      fixture(`${rule}/legacy-runner-commonjs.test.cjs`),
      { [`no-mistakes/${rule}`]: "error" },
      "legacy-runner-commonjs.test.cjs",
      {},
      "commonjs",
    );
    expect(findings.map(({ messageId }) => messageId)).toEqual(["message"]);
  });
  it("does not infer a namespace from repeated declarations", () => {
    expect(check("legacy-runner-namespace-controls.test.js")).toEqual([]);
  });
});
