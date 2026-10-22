import { describe, expect, it } from "vitest";
import { fixture, lint } from "./helpers.mjs";

const DB = { importSpecifier: "@example/db" };
function check(name, rule, options = DB) {
  return lint(
    fixture(`postgres-executor-recall/${name}.ts`),
    {
      [`no-mistakes/postgres-no-${rule}`]: ["error", options],
    },
    "fixture.ts",
  );
}

describe("PostgreSQL same-file executor calls", () => {
  for (const [name, rule, count] of [
    ["manual", "manual-transaction", 9],
    ["registry", "nonliteral-sql-registry-key", 9],
    ["fanout", "unbounded-query-fanout", 10],
  ]) {
    it(`${rule} follows static aliases and one forwarder`, () => {
      const messages = check(name, rule);
      expect(messages).toHaveLength(count);
      expect(
        messages.every((message) => message.ruleId === `no-mistakes/postgres-no-${rule}`),
      ).toBe(true);
      expect(check(name, rule, { executorNames: [] })).toEqual([]);
    });
    it(`${rule} preserves suppression at the forwarded call`, () => {
      expect(check(`suppressed-${rule}`, rule)).toEqual([]);
      expect(check(`file-${rule}`, rule)).toEqual([]);
    });
  }
  it("matches configured call/apply exports before forwarding", () => {
    expect(
      check("configured-methods", "manual-transaction", {
        ...DB,
        executorNames: ["call", "apply"],
      }),
    ).toHaveLength(3);
  });
  it("preserves scoped factory/type executors and their lexical boundaries", () => {
    expect(
      check("scoped", "manual-transaction", {
        ...DB,
        executorFactoryNames: ["makeExecutor"],
        executorTypeNames: ["Executor"],
      }),
    ).toHaveLength(2);
  });
  it("follows namespace aliases, scoped query destructures, and awaited forwarders", () => {
    expect(check("more-static", "manual-transaction")).toHaveLength(9);
  });
  it("does not follow unknown, mutable, shadowed or second-hop calls", () => {
    expect(check("silent", "manual-transaction")).toEqual([]);
    expect(check("registry-silent", "nonliteral-sql-registry-key")).toEqual([]);
    expect(check("fanout-silent", "unbounded-query-fanout")).toEqual([]);
    expect(check("silent", "unbounded-query-fanout")).toEqual([]);
  });
});
