import { describe, expect, it } from "vitest";
import { fixture, lint } from "./helpers.mjs";

const rule = "no-mistakes/postgres-no-unbounded-query-fanout";
function check(name, executorNames) {
  return lint(
    fixture(`postgres-executor-recall/${name}.ts`),
    {
      [rule]: [
        "error",
        { importSpecifier: "@example/db", ...(executorNames && { executorNames }) },
      ],
    },
    "fixture.ts",
  );
}

describe("PostgreSQL fanout static forwarding", () => {
  it("checks call/apply on Promise.all and map with the actual collection receiver", () => {
    const findings = check("fanout-forwarding");
    expect(findings.map(({ line, messageId }) => [line, messageId])).toEqual(
      [2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 15, 17, 19, 20].map((line) => [line, "unboundedFanout"]),
    );
    expect(check("fanout-forwarding", [])).toEqual([]);
  });
  it("preserves line, next-line, and file suppression", () => {
    expect(check("fanout-forwarding-suppressed")).toEqual([]);
    expect(check("fanout-forwarding-file")).toEqual([]);
  });
  it("leaves bounded receivers and unknown forwarding arguments alone", () => {
    expect(check("fanout-forwarding-silent")).toEqual([]);
  });
});
