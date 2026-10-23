import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { fixture, lint, __dirname, plugin } from "./helpers.mjs";

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
const lines = (messages) => messages.map((message) => message.line);

describe(id, () => {
  it("matches imported export names, aliases and namespace calls with binding identity", () => {
    assert.deepEqual(lines(check("usage.ts")), [1, 7, 8, 9, 10]);
    assert.deepEqual(
      lines(
        check("default.mjs", {
          targets: [{ ...target, importedNamePattern: "^(default|paginate-)" }],
        }),
      ),
      [1, 2, 3, 4],
    );
  });
  it("makes lexical call names explicit and independent from origin or shadowing", () => {
    assert.deepEqual(
      lines(check("usage.ts", { targets: [lexical] })),
      [8, 9, 10, 13, 16, 17, 25, 26],
    );
    assert.ok(
      check("usage.ts", { targets: [lexical] }).every(
        (message) => message.message === lexical.message,
      ),
    );
    // A single target matching both selectors emits one call finding.
    assert.deepEqual(
      lines(check("usage.ts", { targets: [{ ...target, ...lexical }] })),
      [1, 7, 8, 9, 10, 13, 16, 17, 25, 26],
    );
  });
  it("is opt-in, supplies a generic message and validates expressions eagerly", () => {
    assert.deepEqual(check("usage.ts", {}), []);
    assert.deepEqual(plugin.rules[id].create({ options: [] }), {});
    assert.match(check("usage.ts")[0].message, /Restricted API 'paginateEntries'.*allowed API/);
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
  it("supports file, line and next-line suppression using actual comments", () => {
    for (const name of ["suppressed.mjs", "file-suppressed.mjs"])
      assert.deepEqual(check(name, { targets: [{ ...target, ...lexical }] }), []);
    assert.deepEqual(lines(check("directives.mjs", { targets: [lexical] })), [3, 5]);
  });
  it("keeps ESLint and Oxlint findings identical", () => {
    const names = [
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
