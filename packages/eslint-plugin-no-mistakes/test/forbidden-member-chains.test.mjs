import assert from "node:assert/strict";
import { resolve } from "node:path";
import { Linter } from "eslint";
import { describe, it } from "vitest";
import { fixture, lint, plugin } from "./helpers.mjs";
import {
  check,
  drawer,
  entry,
  fixtureRoot,
  hits,
  id,
  oxlintFiles,
  panel,
  priv,
  slot,
} from "./forbidden-member-chains-helpers.mjs";
describe(id, () => {
  it("matches reads, calls, literal keys, optionals, and static roots", () => {
    assert.deepEqual(
      hits(check("unsafe.mjs")).map(([line, , , message]) => [line, message]),
      [
        [1, entry],
        [2, entry],
        [3, entry],
        [4, entry],
        [5, entry],
        [6, panel],
        [7, panel],
        [8, panel],
        [9, panel],
        [10, panel],
        [11, panel],
        [12, panel],
        [13, drawer],
        [14, drawer],
        [15, slot],
        [16, slot],
        [17, slot],
        [18, entry],
        [19, entry],
        [20, panel],
        [21, entry],
        [22, entry],
        [22, panel],
        [26, priv],
        [29, panel],
        [32, panel],
        [35, panel],
        [36, panel],
        [37, panel],
        [38, panel],
      ],
    );
    const spans = hits(check("unsafe.mjs"));
    assert.deepEqual(
      spans.filter((hit) => hit[0] === 2 || hit[0] === 11 || hit[0] === 12),
      [
        [2, 1, 16, entry],
        [11, 1, 23, panel],
        [12, 1, 25, panel],
      ],
    );
  });

  it("does not match strings, comments, JSX, unrelated paths, or dynamic keys", () => {
    assert.deepEqual(check("safe.mjs"), []);
    assert.deepEqual(hits(check("markup.jsx")), [[2, 1, 14, entry]]);
  });

  it("reports one outermost access and does not double-report a callee", () => {
    assert.deepEqual(hits(check("nested.mjs")), [
      [1, 1, 28, panel],
      [2, 1, 17, panel],
      [3, 1, 14, entry],
      [4, 1, 19, panel],
      [5, 1, 16, entry],
      [6, 1, 14, entry],
      [7, 1, 31, drawer],
      [7, 1, 17, panel],
      [8, 1, 32, entry],
      [8, 15, 31, panel],
    ]);
  });

  it("ignores transparent wrappers around the same chain", () => {
    assert.deepEqual(
      hits(check("wrappers.ts")).map(([line, , , message]) => [line, message]),
      [
        [1, entry],
        [2, entry],
        [3, entry],
        [4, entry],
        [5, entry],
        [6, panel],
        [7, panel],
        [8, panel],
        [9, panel],
        [10, panel],
        [11, panel],
        [12, panel],
        [13, entry],
      ],
    );
    assert.equal(hits(check("wrappers.ts"))[8][2], 32);
  });

  it("supports receiver forms and literal segments", () => {
    assert.deepEqual(
      hits(
        check("named.mjs", {
          chains: [{ receiver: "records", path: "*.entry", message: "Named." }],
        }),
      ),
      [
        [1, 1, 14, "Named."],
        [3, 1, 16, "Named."],
      ],
    );
    assert.deepEqual(
      hits(
        check("named.mjs", {
          chains: [
            { path: "records.entry", message: "Exact." },
            { path: "*.entry", message: "Wildcard." },
          ],
        }),
      ).map(([, , , message]) => message),
      ["Exact.", "Wildcard.", "Wildcard.", "Exact.", "Wildcard."],
    );
    assert.deepEqual(
      hits(
        check("literal-name.mjs", {
          chains: [{ receiver: "*", path: ["panel.open"], message: "Literal." }],
        }),
      ),
      [[1, 1, 20, "Literal."]],
    );
    assert.deepEqual(
      hits(
        check("literal-name.mjs", {
          chains: [{ receiver: "*", path: "*.panel.open", message: "Any." }],
        }),
      ),
      [[2, 1, 17, "Any."]],
    );
    assert.deepEqual(check("named.mjs", { chains: [] }), []);
    assert.deepEqual(
      lint(
        fixture(`${id}/named.mjs`),
        { [`no-mistakes/${id}`]: "error" },
        resolve(fixtureRoot, "named.mjs"),
      ),
      [],
    );
  });

  it("treats chains deeper than the walk as unproven", () => {
    assert.deepEqual(hits(check("bounds.mjs")), [[3, 1, 127, panel]]);
  });

  it("supports messages and no-mistakes suppression without a reason", () => {
    assert.equal(
      check("unsafe.mjs", { chains: [{ path: "records.entry", message: "Custom." }] })[0].message,
      "Custom.",
    );
    assert.deepEqual(
      hits(check("suppressions.mjs")).map(([line]) => line),
      [5, 9],
    );
    assert.deepEqual(check("suppression-file.mjs"), []);
  });

  it("takes file scope from the lint config", () => {
    const linter = new Linter({ configType: "flat", cwd: fixtureRoot });
    const config = {
      files: ["scoped.mjs"],
      languageOptions: { ecmaVersion: 2024, sourceType: "module" },
      plugins: { "no-mistakes": plugin },
      rules: {
        [`no-mistakes/${id}`]: ["error", { chains: [{ path: "records.entry", message: entry }] }],
      },
    };
    assert.equal(
      linter.verify(fixture(`${id}/scoped.mjs`), config, {
        filename: resolve(fixtureRoot, "scoped.mjs"),
      }).length,
      1,
    );
    assert.deepEqual(
      linter.verify(fixture(`${id}/unscoped.mjs`), config, {
        filename: resolve(fixtureRoot, "unscoped.mjs"),
      }),
      [],
    );
    const scope = oxlintFiles(["scoped.mjs", "unscoped.mjs"], ".oxlintrc.scope.json");
    assert.equal(scope.get("scoped.mjs").length, 1);
    assert.equal(scope.get("unscoped.mjs").length, 0);
  });

  it("matches oxlint jsPlugins on the saved fixtures", () => {
    const names = [
      "unsafe.mjs",
      "safe.mjs",
      "nested.mjs",
      "wrappers.ts",
      "suppressions.mjs",
      "suppression-file.mjs",
      "bounds.mjs",
      "literal-name.mjs",
      "named.mjs",
      "markup.jsx",
    ];
    const diagnostics = oxlintFiles(names);
    for (const name of names) {
      assert.deepEqual(
        diagnostics.get(name).map((diagnostic) => diagnostic.message),
        check(name).map((message) => message.message),
        name,
      );
    }
  });

  it("exports an opt-in rule without an unsafe fix", () => {
    assert.ok(plugin.rules[id]);
    assert.equal(plugin.rules[id].meta.docs.recommended, false);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${id}`], undefined);
    assert.equal(plugin.configs.strict.rules[`no-mistakes/${id}`], undefined);
    assert.equal(plugin.rules[id].meta.fixable, undefined);
  });
});
