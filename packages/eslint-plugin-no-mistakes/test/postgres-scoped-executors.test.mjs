import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, messages, require } from "./helpers.mjs";

const {
  collectScopedExecutors,
  isScopedExecutor,
} = require("../src/rules/postgres-scoped-executors");

const MANUAL = "postgres-no-manual-transaction";
const FANOUT = "postgres-no-unbounded-query-fanout";
const SCOPED = {
  importSpecifier: "@example/db",
  executorFactoryNames: ["openTransaction"],
  executorTypeNames: ["TxExecutor"],
};

function manual(name, option = SCOPED) {
  return messages(fixture(`postgres-scoped/${name}`), MANUAL, option, "app.ts");
}

describe("scoped PostgreSQL executors", () => {
  it("scans factory results for every block-scoped declaration kind", () => {
    // const (awaited and not), let, using, await using, and a `.query` member.
    assert.equal(manual("factory-decls.ts").length, 6);
  });

  it("scans typed parameters, optional parameters, and inline property types", () => {
    // plain, optional, arrow, union, defaulted, destructured, destructured default.
    assert.equal(manual("typed-params.ts").length, 7);
  });

  it("scans an aliased inline `type` specifier", () => {
    assert.equal(manual("inline-type-specifier.ts").length, 1);
  });

  it("does not match same-named identifiers outside the declaring scope", () => {
    assert.equal(manual("negatives.ts").length, 2);
  });

  it("does not match imports from another module", () => {
    assert.deepEqual(manual("different-module.ts"), []);
  });

  it("matches factory and type imports from subpaths of importSpecifier", () => {
    // type import, inline `type` specifier, and a subpath factory import.
    assert.equal(manual("subpath-imports.ts").length, 3);
  });

  it("does not match modules that only share a string prefix", () => {
    assert.deepEqual(manual("lookalike-modules.ts"), []);
  });

  it("separates value type imports from type-only factory imports", () => {
    assert.equal(manual("value-type-import.ts").length, 1);
  });

  it("matches from any module when importSpecifier is empty", () => {
    const option = { ...SCOPED, importSpecifier: "", executorNames: [] };
    assert.equal(manual("any-module.ts", option).length, 2);
  });

  it("leaves findings unchanged when both options are absent", () => {
    const baseline = { importSpecifier: "@example/db" };
    assert.equal(manual("defaults.ts", baseline).length, 1);
    assert.equal(manual("defaults.ts").length, 3);
  });

  it("applies to the fan-out rule", () => {
    const run = (option) =>
      messages(fixture("postgres-scoped/fanout.ts"), FANOUT, option, "app.ts");
    assert.deepEqual(run(SCOPED), ["unboundedFanout"]);
    assert.deepEqual(run({ importSpecifier: "@example/db" }), []);
  });

  it("exposes scope lookups as range checks", () => {
    assert.equal(collectScopedExecutors({ body: [] }, SCOPED_DEFAULTS).length, 0);
    assert.equal(isScopedExecutor(undefined, "tx", { range: [1, 2] }), false);
    const scoped = [{ name: "tx", range: [10, 20] }];
    assert.equal(isScopedExecutor(scoped, "tx", { range: [10, 12] }), true);
    assert.equal(isScopedExecutor(scoped, "tx", { range: [20, 22] }), false);
    assert.equal(isScopedExecutor(scoped, "run", { range: [12, 14] }), false);
  });
});

const SCOPED_DEFAULTS = {
  importSpecifier: "",
  executorFactoryNames: [],
  executorTypeNames: [],
};
