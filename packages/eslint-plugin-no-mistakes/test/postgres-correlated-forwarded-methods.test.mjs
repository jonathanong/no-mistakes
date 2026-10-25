import { describe, expect, it } from "vitest";
import { fixture, lint } from "./helpers.mjs";

const rule = "no-mistakes/postgres-no-correlated-sql-builder";
function check(name, builderMethods = ["append"]) {
  return lint(
    fixture(`postgres-no-correlated-sql-builder/${name}.ts`),
    {
      [rule]: ["error", { builderMethods, tagNames: ["sql"] }],
    },
    "fixture.ts",
  );
}
describe("correlated SQL explicit receiver forwarding", () => {
  it("retains apply, const aliases, comma methods and transparent wrappers", () => {
    const findings = check("forwarded-methods");
    expect(findings.map(({ line, messageId }) => [line, messageId])).toEqual(
      [3, 9, 13, 17, 21].map((line) => [line, "correlated"]),
    );
    expect(findings.at(-1).message).toContain("appends on other");
  });
  it("keeps every builder per guard without counting one guard twice", () => {
    const findings = check("forwarded-method-multiple-builders");
    expect(findings.map(({ line }) => line)).toEqual([3, 11, 18, 18]);
    expect(findings.slice(0, 2).every(({ message }) => message.includes("appends on q"))).toBe(
      true,
    );
    expect(findings.slice(2).map(({ message }) => message.includes("appends on other"))).toEqual([
      true,
      false,
    ]);
  });
  it("keeps unknown arguments, mutable bindings and split receivers silent", () => {
    expect(check("forwarded-method-controls")).toEqual([]);
  });
  it("prioritizes configured methods named apply or call", () => {
    expect(check("forwarded-method-priority", ["apply", "call"]).map(({ line }) => line)).toEqual([
      2, 6,
    ]);
  });
  it("honors suppression on the guarded append", () => {
    expect(check("forwarded-method-suppression")).toEqual([]);
    expect(check("forwarded-method-file-suppression")).toEqual([]);
  });
});
