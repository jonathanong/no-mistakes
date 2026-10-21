import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, fixture, messages, require } from "./helpers.mjs";

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

  it("scans a relative import that resolves inside the importSpecifier package", () => {
    const name = "postgres-scoped/relative-inside/orders.ts";
    assert.equal(messages(fixture(name), MANUAL, SCOPED, fixtureFile(name)).length, 1);
  });

  it("does not scan a relative import that does not resolve to a file", () => {
    const name = "postgres-scoped/relative-inside/orders-missing.ts";
    assert.deepEqual(messages(fixture(name), MANUAL, SCOPED, fixtureFile(name)), []);
  });

  it("does not scan a relative import that resolves outside that package", () => {
    // The file sits in @example/db, but the import lands in another directory.
    const name = "postgres-scoped/relative-inside/orders-external.ts";
    assert.deepEqual(messages(fixture(name), MANUAL, SCOPED, fixtureFile(name)), []);
  });

  it("does not scan a relative import when the package root cannot be determined", () => {
    const name = "postgres-scoped/relative-outside/orders.ts";
    assert.deepEqual(messages(fixture(name), MANUAL, SCOPED, fixtureFile(name)), []);
    assert.deepEqual(messages(fixture(name), MANUAL, SCOPED, "app.ts"), []);
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

  it("scans quoted destructured parameter keys", () => {
    assert.equal(manual("quoted-keys.ts").length, 2);
  });

  it("matches a string-literal import specifier name", () => {
    assert.equal(manual("literal-import.ts").length, 1);
  });

  it("does not treat a relative import as in-package when package.json is unreadable", () => {
    // broken-json/package.json is intentionally truncated so JSON.parse throws.
    const name = "postgres-scoped/broken-json/orders.ts";
    assert.deepEqual(messages(fixture(name), MANUAL, SCOPED, fixtureFile(name)), []);
  });

  it("does not treat a relative import as in-package when package.json name is not a string", () => {
    const name = "postgres-scoped/numeric-name/orders.ts";
    assert.deepEqual(messages(fixture(name), MANUAL, SCOPED, fixtureFile(name)), []);
  });

  it("does not scan a relative import that only finds the package under node_modules", () => {
    // App-local ./transaction is not inside node_modules/@example/db.
    const name = "postgres-scoped/nested-modules/orders.ts";
    assert.deepEqual(messages(fixture(name), MANUAL, SCOPED, fixtureFile(name)), []);
  });

  it("scans a relative import that resolves inside a nested node_modules package", () => {
    // The specifier walks into node_modules/@example/db, so the file is in-package.
    const name = "postgres-scoped/nested-modules/orders-inside.ts";
    assert.equal(messages(fixture(name), MANUAL, SCOPED, fixtureFile(name)).length, 1);
  });

  it("does not resolve a relative import when importSpecifier is not a package name", () => {
    // Each directory's package.json name is what packageName() would return
    // without the matching guard, so that guard is the only rejection.
    // A leading "/" needs no fixture: split("/")[0] is empty and `name || null`
    // already returns null. A non-string package.json name needs none either:
    // packageRoot only compares it with a string specifier.
    assert.deepEqual(scopedPackage("package-name-slash", "@/db"), []);
    assert.deepEqual(scopedPackage("package-name-empty", "@scope/"), []);
    assert.deepEqual(scopedPackage("package-name-dot", "./db"), []);
  });

  it("does not treat an empty filename as the package root", () => {
    const cwd = fixtureFile("postgres-scoped/empty-filename/nested");
    const script = resolve(__dirname, "postgres-scoped-empty-filename.mjs");
    const child = spawnSync(process.execPath, [script, cwd, ""], { encoding: "utf8" });
    assert.equal(child.status, 0, child.stderr);
    assert.equal(child.stdout, "0");
  });

  it("exposes scope lookups as range checks", () => {
    assert.equal(collectScopedExecutors({ body: [] }, SCOPED_DEFAULTS).length, 0);
    assert.equal(collectScopedExecutors({ body: [] }, SCOPED, "").length, 0);
    assert.equal(isScopedExecutor(undefined, "tx", { range: [1, 2] }), false);
    const scoped = [{ name: "tx", range: [10, 20] }];
    assert.equal(isScopedExecutor(scoped, "tx", { range: [10, 12] }), true);
    assert.equal(isScopedExecutor(scoped, "tx", { range: [20, 22] }), false);
    assert.equal(isScopedExecutor(scoped, "run", { range: [12, 14] }), false);
  });
});

function fixtureFile(name) {
  return resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", name);
}

function scopedPackage(directory, importSpecifier) {
  const name = `postgres-scoped/${directory}/orders.ts`;
  return messages(fixture(name), MANUAL, { ...SCOPED, importSpecifier }, fixtureFile(name));
}

const SCOPED_DEFAULTS = {
  importSpecifier: "",
  executorFactoryNames: [],
  executorTypeNames: [],
};
