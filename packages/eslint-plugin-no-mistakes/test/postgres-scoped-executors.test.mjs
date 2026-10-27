import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture, require } from "./helpers.mjs";

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
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function fixtureFile(name) {
  return resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", name);
}

function scopedCase(name, option = SCOPED, filename = "app.ts") {
  const path = name.startsWith("postgres-scoped/") ? name : `postgres-scoped/${name}`;
  return {
    code: fixture(path),
    filename: filename === "app.ts" ? "app.ts" : fixtureFile(path),
    options: [option],
    languageOptions: { parser: tsParser },
  };
}

const manual = {
  valid: [
    // does not match imports from another module.
    // Zero findings.
    scopedCase("different-module.ts"),
    // does not match modules that only share a string prefix.
    // Zero findings.
    scopedCase("lookalike-modules.ts"),
    // does not scan a relative import that does not resolve to a file.
    // Zero findings.
    scopedCase("relative-inside/orders-missing.ts", SCOPED, "absolute"),
    // does not scan a relative import that resolves outside that package.
    // The file sits in @example/db, but the import lands in another directory.
    // Zero findings.
    scopedCase("relative-inside/orders-external.ts", SCOPED, "absolute"),
    // does not scan a relative import when the package root cannot be determined.
    // Zero findings.
    scopedCase("relative-outside/orders.ts", SCOPED, "absolute"),
    // Zero findings.
    scopedCase("relative-outside/orders.ts"),
    // does not treat a relative import as in-package when package.json is unreadable.
    // broken-json/package.json is intentionally truncated so JSON.parse throws.
    // Zero findings.
    scopedCase("broken-json/orders.ts", SCOPED, "absolute"),
    // does not treat a relative import as in-package when package.json name is not a string.
    // Zero findings.
    scopedCase("numeric-name/orders.ts", SCOPED, "absolute"),
    // does not scan a relative import that only finds the package under node_modules.
    // App-local ./transaction is not inside node_modules/@example/db.
    // Zero findings.
    scopedCase("nested-modules/orders.ts", SCOPED, "absolute"),
    // does not resolve a relative import when importSpecifier is not a package name.
    // Each directory's package.json name is what packageName() would return
    // without the matching guard, so that guard is the only rejection.
    // A leading "/" needs no fixture: split("/")[0] is empty and `name || null`
    // already returns null. A non-string package.json name needs none either:
    // packageRoot only compares it with a string specifier.
    // Zero findings.
    scopedCase("package-name-slash/orders.ts", { ...SCOPED, importSpecifier: "@/db" }, "absolute"),
    // Zero findings.
    scopedCase(
      "package-name-empty/orders.ts",
      { ...SCOPED, importSpecifier: "@scope/" },
      "absolute",
    ),
    // Zero findings.
    scopedCase("package-name-dot/orders.ts", { ...SCOPED, importSpecifier: "./db" }, "absolute"),
  ],
  invalid: [
    // scans factory results for every block-scoped declaration kind.
    // const (awaited and not), let, using, await using, and a `.query` member.
    // Six findings.
    scopedCase("factory-decls.ts"),
    // scans typed parameters, optional parameters, and inline property types.
    // plain, optional, arrow, union, defaulted, destructured, destructured default.
    // Seven findings.
    scopedCase("typed-params.ts"),
    // scans an aliased inline `type` specifier.
    // One finding.
    scopedCase("inline-type-specifier.ts"),
    // does not match same-named identifiers outside the declaring scope.
    // Two findings.
    scopedCase("negatives.ts"),
    // matches factory and type imports from subpaths of importSpecifier.
    // type import, inline `type` specifier, and a subpath factory import.
    // Three findings.
    scopedCase("subpath-imports.ts"),
    // separates value type imports from type-only factory imports.
    // One finding.
    scopedCase("value-type-import.ts"),
    // scans a relative import that resolves inside the importSpecifier package.
    // One finding.
    scopedCase("relative-inside/orders.ts", SCOPED, "absolute"),
    // matches from any module when importSpecifier is empty.
    // Two findings.
    scopedCase("any-module.ts", { ...SCOPED, importSpecifier: "", executorNames: [] }),
    // leaves findings unchanged when both options are absent.
    // One finding.
    scopedCase("defaults.ts", { importSpecifier: "@example/db" }),
    // Three findings.
    scopedCase("defaults.ts"),
    // scans quoted destructured parameter keys.
    // Two findings.
    scopedCase("quoted-keys.ts"),
    // matches a string-literal import specifier name.
    // One finding.
    scopedCase("literal-import.ts"),
    // scans a relative import that resolves inside a nested node_modules package.
    // The specifier walks into node_modules/@example/db, so the file is in-package.
    // One finding.
    scopedCase("nested-modules/orders-inside.ts", SCOPED, "absolute"),
  ],
};

const fanout = {
  valid: [
    // applies to the fan-out rule.
    // Zero findings.
    scopedCase("fanout.ts", { importSpecifier: "@example/db" }),
  ],
  invalid: [
    // One unboundedFanout finding.
    scopedCase("fanout.ts"),
  ],
};

it("snapshots scoped PostgreSQL executor diagnostics", async () => {
  await expect(
    snapshotDocument({
      [MANUAL]: snapshotSuite(MANUAL, languageOptions, manual),
      [FANOUT]: snapshotSuite(FANOUT, languageOptions, fanout),
    }),
  ).toMatchFileSnapshot("snapshots/postgres-scoped-executors.snap");
}, 30_000);

describe("scoped PostgreSQL executors", () => {
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

const SCOPED_DEFAULTS = {
  importSpecifier: "",
  executorFactoryNames: [],
  executorTypeNames: [],
};
