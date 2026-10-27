import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";
const rule = "no-consecutive-unbatched-reads";
const options = { clients: ["client"], readMethods: ["get", "ttl"] };
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture/client-contracts");
const expectedLines = [4, 7, 10, 17, 21, 24, 24, 25, 28, 28];
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parser: tsParser,
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const diagnosticCase = (name, config = options) => ({
  code: fixture(`client-contracts/${name}.ts`),
  filename: "fixture.ts",
  options: [config],
});
describe("configured read statement shapes", () => {
  it("snapshots statement-shape and suppression diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, {
          valid: [
            diagnosticCase("reads-statement-boundaries"),
            diagnosticCase("reads-statements-suppressed"),
            diagnosticCase("reads-statements-suppressed-file"),
          ],
          invalid: [
            diagnosticCase("reads-statements"),
            diagnosticCase("reads-statements", {
              imports: [{ source: "@app/cache", imported: "cache" }],
              readMethods: ["get", "ttl"],
            }),
            diagnosticCase("reads-declarator-suppression-control"),
          ],
        }),
      }),
    ).toMatchFileSnapshot("snapshots/read-statement-shapes.snap");
  });
  it("matches ESLint findings and suppression locations through Oxlint", () => {
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        resolve(root, ".oxlintrc.reads.json"),
        "--format",
        "json",
        ...[
          "reads-statements",
          "reads-declarator-suppression-control",
          "reads-statement-boundaries",
          "reads-statements-suppressed",
          "reads-statements-suppressed-file",
        ].map((name) => resolve(root, `${name}.ts`)),
      ],
      { encoding: "utf8" },
    );
    assert.equal(result.error, undefined);
    assert.equal(result.status, 1, result.stdout + result.stderr);
    const diagnostics = JSON.parse(result.stdout).diagnostics.filter(({ code }) =>
      code.includes(rule),
    );
    assert.deepEqual(
      diagnostics
        .map(({ filename, labels }) => [filename.split("/").at(-1), labels[0].span.line])
        .sort((a, b) => a[0].localeCompare(b[0]) || a[1] - b[1]),
      [
        ...[5, 6].map((line) => ["reads-declarator-suppression-control.ts", line]),
        ...expectedLines.map((line) => ["reads-statements.ts", line]),
      ],
    );
  });
});
