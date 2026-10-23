import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, fixture, lint } from "./helpers.mjs";
const rule = "no-consecutive-unbatched-reads";
const options = { clients: ["client"], readMethods: ["get", "ttl"] };
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture/client-contracts");
const results = (name, config = options) =>
  lint(
    fixture(`client-contracts/${name}.ts`),
    {
      [`no-mistakes/${rule}`]: ["error", config],
    },
    "fixture.ts",
  );
const expectedLines = [4, 7, 10, 17, 21, 24, 24, 25, 28, 28];
describe("configured read statement shapes", () => {
  it("reports assignments, exports, returns, cases, and sequential declarators at the second read", () => {
    const diagnostics = results("reads-statements");
    assert.deepEqual(
      diagnostics.map(({ messageId }) => messageId),
      Array(10).fill("unbatchedReads"),
    );
    assert.deepEqual(
      diagnostics.map(({ line }) => line),
      expectedLines,
    );
  });
  it("uses the same shapes for configured runtime imports", () => {
    assert.deepEqual(
      results("reads-statements", {
        imports: [{ source: "@app/cache", imported: "cache" }],
        readMethods: ["get", "ttl"],
      }).map(({ line }) => line),
      expectedLines,
    );
  });
  it("preserves assignment and declarator dependencies and sequence boundaries", () => {
    assert.deepEqual(results("reads-statement-boundaries"), []);
  });
  it("honors file, line, and next-line suppressions at the newly supported locations", () => {
    for (const name of ["reads-statements-suppressed", "reads-statements-suppressed-file"])
      assert.deepEqual(results(name), []);
  });
  it("keeps individual-declarator suppression local to that read", () => {
    assert.deepEqual(
      results("reads-declarator-suppression-control").map(({ line }) => line),
      [5, 6],
    );
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
