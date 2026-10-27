import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, __dirname, lint, plugin } from "./helpers.mjs";

const id = "restricted-api-usage";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
const target = { source: "@example/client", importedNamePattern: "^paginate[A-Z]" };
const lexical = { callNamePattern: "^paginate[A-Z]", message: "Use explicit continuation." };
function check(name, options = { targets: [target] }) {
  return lint(
    fixture(`${id}/${name}`),
    { [`no-mistakes/${id}`]: ["error", options] },
    resolve(root, name),
  );
}
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
function snapshotCase(name, options = { targets: [target] }) {
  return {
    code: fixture(`${id}/${name}`),
    filename: resolve(root, name),
    options: [options],
    ...(name.endsWith(".ts") ? { languageOptions: { parser: tsParser } } : {}),
  };
}

describe(id, () => {
  it("snapshots restricted API diagnostics", async () => {
    const both = { targets: [{ ...target, ...lexical }] };
    await expect(
      snapshotDocument({
        [id]: snapshotSuite(id, languageOptions, {
          valid: [
            snapshotCase("usage.ts", {}),
            ...[
              "suppressed.mjs",
              "file-suppressed.mjs",
              "recall-suppressed.ts",
              "recall-file-suppressed.ts",
            ].map((name) => snapshotCase(name, { targets: [{ ...target, ...lexical }] })),
          ],
          invalid: [
            snapshotCase("usage.ts"),
            snapshotCase("default.mjs", {
              targets: [{ ...target, importedNamePattern: "^(default|paginate-)" }],
            }),
            snapshotCase("recall.ts"),
            snapshotCase("recall.ts", {
              targets: [{ ...target, importedNamePattern: "^paginate-" }],
            }),
            snapshotCase("usage.ts", { targets: [lexical] }),
            snapshotCase("usage.ts", both),
            snapshotCase("directives.mjs", { targets: [lexical] }),
          ],
        }),
      }),
    ).toMatchFileSnapshot("snapshots/restricted-api-usage.snap");
  });
  it("validates expressions eagerly", () => {
    assert.deepEqual(plugin.rules[id].create({ options: [] }), {});
    for (const key of ["importedNamePattern", "callNamePattern"]) {
      assert.throws(
        () => check("usage.ts", { targets: [{ ...target, [key]: "[" }] }),
        /Invalid regular expression/,
      );
    }
    for (const partial of [{ source: "@example/client" }, { importedNamePattern: "x" }]) {
      assert.throws(
        () => check("usage.ts", { targets: [{ ...partial, ...lexical }] }),
        /configuration|match|propert/i,
      );
    }
    assert.throws(
      () => check("usage.ts", { targets: [{ source: "@example/client" }] }),
      /configuration|match|propert/i,
    );
  });
  it("keeps ESLint and Oxlint findings identical", () => {
    const names = [
      "recall.ts",
      "recall-suppressed.ts",
      "recall-file-suppressed.ts",
      "usage.ts",
      "default.mjs",
      "suppressed.mjs",
      "file-suppressed.mjs",
      "directives.mjs",
    ];
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        resolve(root, ".oxlintrc.json"),
        "--format",
        "json",
        ...names.map((name) => resolve(root, name)),
      ],
      { encoding: "utf8" },
    );
    assert.equal(result.error, undefined, result.stderr);
    const diagnostics = JSON.parse(result.stdout).diagnostics.filter((message) =>
      message.code?.includes(id),
    );
    for (const name of names) {
      const expected = check(name, { targets: [{ ...target, ...lexical }] });
      const actual = diagnostics.filter((message) => message.filename.endsWith(`/${name}`));
      assert.deepEqual(
        actual.map((message) => [message.labels[0].span.line, message.message]),
        expected.map((message) => [message.line, message.message]),
      );
    }
  });
});
