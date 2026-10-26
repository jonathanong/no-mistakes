import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, fixture, lint, messages, plugin } from "./helpers.mjs";

const rule = "postgres-no-async-sql-builder";
const options = { builders: [{ names: ["build"] }] };
const holes = () => fixture(`${rule}/ast-holes.ts`);

describe("plugin AST null holes", () => {
  it("does not crash or report on sparse patterns, arrays, and optional children", () => {
    assert.deepEqual(messages(holes(), rule, options, "holes.ts"), []);
  });

  it("continues collecting types and checking builders after a hole", () => {
    assert.deepEqual(
      messages(fixture(`${rule}/ast-holes-builder.ts`), rule, options, "builder.ts"),
      ["removeAsync"],
    );
  });

  it("exercises the sparse fixture with every plugin rule", () => {
    for (const name of Object.keys(plugin.rules)) {
      // Default options exercise general walkers; SQL type collection needs a builder scope.
      const schema = plugin.rules[name].meta.schema[0];
      const config =
        name === rule
          ? ["error", options]
          : schema?.properties?.executorNames
            ? ["error", { executorNames: ["query"] }]
            : "error";
      const findings = lint(holes(), { [`no-mistakes/${name}`]: config }, "holes.ts");
      assert.equal(
        findings.some((finding) => finding.fatal),
        false,
        name,
      );
    }
  });

  it("checks the sparse fixtures with real Oxlint", () => {
    const root = resolve(
      __dirname,
      "../../../test-cases/eslint-snippets/fixture/postgres-no-async-sql-builder",
    );
    for (const [name, lines] of [
      ["ast-holes.ts", []],
      ["ast-holes-builder.ts", [4]],
    ]) {
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, ".oxlintrc-holes.json"),
          "--format",
          "json",
          resolve(root, name),
        ],
        { encoding: "utf8" },
      );
      assert.equal(result.status, lines.length ? 1 : 0, result.stderr);
      assert.deepEqual(
        JSON.parse(result.stdout)
          .diagnostics.filter((diagnostic) => diagnostic.code.includes(rule))
          .map((diagnostic) => diagnostic.labels[0].span.line),
        lines,
      );
    }
  });
});
