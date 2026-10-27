import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import {
  oxlintFindings,
  pluginFindings,
  snapshotCases,
  snapshotDocument,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture, lint } from "./helpers.mjs";

const id = "test-no-error-message-matching";
const rule = `no-mistakes/${id}`;

function sourceTypeOf(name) {
  return /\.(cjs|cts)$/.test(name) ? "commonjs" : name.endsWith(".js") ? "script" : "module";
}

function check(name, filename = name) {
  return lint(fixture(`${id}/${name}`), { [rule]: "error" }, filename, {}, sourceTypeOf(name));
}

function capturedCase(name, fixtureName, filename = fixtureName) {
  const sourceType = sourceTypeOf(fixtureName);
  return {
    name,
    file: filename,
    rule: id,
    ...(sourceType !== "module" ? { sourceType } : {}),
    findings: pluginFindings(
      fixture(`${id}/${fixtureName}`),
      id,
      undefined,
      filename,
      {},
      sourceType,
    ),
  };
}

it("snapshots error-message-captured-values diagnostics", async () => {
  const cases = [
    capturedCase(
      "reports immutable captures, static keys, wrappers and alias chains at their assertions",
      "captured-values.ts",
    ),
    capturedCase(
      "preserves identity through erased satisfies wrappers: captured-satisfies.ts",
      "captured-satisfies.ts",
    ),
    capturedCase(
      "preserves identity through erased satisfies wrappers: captured-satisfies-rewrite.ts",
      "captured-satisfies-rewrite.ts",
    ),
    capturedCase(
      "keeps captures message-derived after source binding or property changes",
      "captured-source-change.ts",
    ),
    ...[
      "captured-controls.ts",
      "captured-order.ts",
      "captured-with.js",
      "captured-rewrites.ts",
      "captured-source-rewrites.ts",
      "captured-eval.cts",
      "captured-cjs-rewrite.cjs",
    ].map((file) =>
      capturedCase(
        `preserves mutable, shadowed, dynamic, body and unsupported value boundaries: ${file}`,
        file,
      ),
    ),
    capturedCase(
      "does not confuse direct eval bindings with outer CommonJS captures",
      "captured-eval.cjs",
    ),
    ...["captured-suppressed.ts", "captured-suppressed-file.ts"].map((file) =>
      capturedCase(`preserves file, line, next-line and filename exclusions: ${file}`, file),
    ),
    capturedCase(
      "preserves file, line, next-line and filename exclusions: static-code-analysis",
      "captured-values.ts",
      "static-code-analysis/captured.ts",
    ),
    capturedCase(
      "preserves file, line, next-line and filename exclusions: generated",
      "captured-values.ts",
      "captured.generated.ts",
    ),
  ];

  const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
  for (const file of [
    "captured-values.ts",
    "captured-satisfies.ts",
    "captured-satisfies-rewrite.ts",
    "captured-source-change.ts",
    "captured-rewrites.ts",
    "captured-source-rewrites.ts",
    "captured-eval.cjs",
    "captured-eval.cts",
    "captured-cjs-rewrite.cjs",
    "captured-controls.ts",
    "captured-order.ts",
    "captured-with.js",
    "captured-suppressed.ts",
    "captured-suppressed-file.ts",
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
    // pluginFindings sorts; this parity check keeps both engines' original order.
    const eslint = check(file);
    assert.equal(result.status, eslint.length ? 1 : 0, `${file}: ${result.stderr}`);
    const diagnostics = JSON.parse(result.stdout).diagnostics;
    const relativeFile = `${id}/${file}`;
    cases.push({
      name: `matches real Oxlint for captures, controls and suppressions: ${file}`,
      file: relativeFile,
      rule: id,
      findings: oxlintFindings(diagnostics, relativeFile),
    });
    assert.deepEqual(
      diagnostics.map(({ code, labels }) => [code, labels[0].span.line]),
      eslint.map(({ line }) => [`no-mistakes(${id})`, line]),
      file,
    );
  }

  await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
    "snapshots/error-message-captured-values.snap",
  );
}, 20_000);
