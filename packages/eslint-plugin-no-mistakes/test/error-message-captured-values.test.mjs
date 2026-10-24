import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { fixture, lint, __dirname } from "./helpers.mjs";

const id = "test-no-error-message-matching";
const rule = `no-mistakes/${id}`;
function check(name, filename = name) {
  return lint(
    fixture(`${id}/${name}`),
    { [rule]: "error" },
    filename,
    {},
    /\.(cjs|cts)$/.test(name) ? "commonjs" : name.endsWith(".js") ? "script" : "module",
  );
}

describe("captured error-message values", () => {
  it("reports immutable captures, static keys, wrappers and alias chains at their assertions", () => {
    assert.deepEqual(
      check("captured-values.ts").map(({ line, messageId }) => [line, messageId]),
      [
        [4, "message"],
        [6, "message"],
        [9, "message"],
        [11, "message"],
        [14, "message"],
        [15, "message"],
        [16, "message"],
      ],
    );
  });

  it("preserves identity through erased satisfies wrappers", () => {
    assert.deepEqual(
      check("captured-satisfies.ts").map(({ line, messageId }) => [line, messageId]),
      [5, 7, 8, 10, 11].map((line) => [line, "message"]),
    );
    assert.deepEqual(check("captured-satisfies-rewrite.ts"), []);
  });

  it("keeps captures message-derived after source binding or property changes", () => {
    assert.deepEqual(
      check("captured-source-change.ts").map(({ line, messageId }) => [line, messageId]),
      [
        [6, "message"],
        [9, "message"],
        [13, "message"],
      ],
    );
  });

  it("preserves mutable, shadowed, dynamic, body and unsupported value boundaries", () => {
    for (const name of [
      "captured-controls.ts",
      "captured-order.ts",
      "captured-with.js",
      "captured-rewrites.ts",
      "captured-source-rewrites.ts",
      "captured-eval.cts",
      "captured-cjs-rewrite.cjs",
    ]) {
      assert.deepEqual(check(name), [], name);
    }
  });

  it("does not confuse direct eval bindings with outer CommonJS captures", () => {
    assert.deepEqual(
      check("captured-eval.cjs").map(({ line }) => line),
      [4],
    );
  });

  it("preserves file, line, next-line and filename exclusions", () => {
    for (const name of ["captured-suppressed.ts", "captured-suppressed-file.ts"]) {
      assert.deepEqual(check(name), [], name);
    }
    assert.deepEqual(check("captured-values.ts", "static-code-analysis/captured.ts"), []);
    assert.deepEqual(check("captured-values.ts", "captured.generated.ts"), []);
  });

  it("matches real Oxlint for captures, controls and suppressions", () => {
    const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
    for (const [file, count] of [
      ["captured-values.ts", 7],
      ["captured-satisfies.ts", 5],
      ["captured-satisfies-rewrite.ts", 0],
      ["captured-source-change.ts", 3],
      ["captured-rewrites.ts", 0],
      ["captured-source-rewrites.ts", 0],
      ["captured-eval.cjs", 1],
      ["captured-eval.cts", 0],
      ["captured-cjs-rewrite.cjs", 0],
      ["captured-controls.ts", 0],
      ["captured-order.ts", 0],
      ["captured-with.js", 0],
      ["captured-suppressed.ts", 0],
      ["captured-suppressed-file.ts", 0],
    ]) {
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, ".oxlintrc-captured.json"),
          "--format",
          "json",
          resolve(root, file),
        ],
        { encoding: "utf8" },
      );
      assert.equal(result.status, count ? 1 : 0, `${file}: ${result.stderr}`);
      assert.deepEqual(
        JSON.parse(result.stdout).diagnostics.map(({ code, labels }) => [
          code,
          labels[0].span.line,
        ]),
        check(file).map(({ line }) => [`no-mistakes(${id})`, line]),
        file,
      );
    }
  }, 20_000);
});
