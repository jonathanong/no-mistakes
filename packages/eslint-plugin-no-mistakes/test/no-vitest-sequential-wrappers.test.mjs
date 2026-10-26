import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, fixture, lint, messages } from "./helpers.mjs";

describe("no-vitest-sequential typed wrappers", () => {
  it("reports every wrapped serial call once at its source line", () => {
    const code = fixture("no-vitest-sequential/typed-wrappers.test.ts");
    const findings = lint(code, { "no-mistakes/no-vitest-sequential": "error" }, "typed.test.ts");
    assert.deepEqual(
      findings.map(({ messageId, line }) => [messageId, line]),
      [4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 15, 16, 17, 18, 19, 21].map((line) => [
        "sequential",
        line,
      ]),
    );
  });

  it("preserves dynamic, non-sequential, shadowed and suppressed cases", () => {
    assert.deepEqual(
      messages(
        fixture("no-vitest-sequential/typed-wrappers-valid.test.ts"),
        "no-vitest-sequential",
        undefined,
        "typed.test.ts",
      ),
      [],
    );
  });

  it("matches real Oxlint findings for the saved fixtures", () => {
    const root = resolve(
      __dirname,
      "../../../test-cases/eslint-snippets/fixture/no-vitest-sequential",
    );
    for (const name of ["typed-wrappers.test.ts", "typed-wrappers-valid.test.ts"]) {
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, ".oxlintrc-wrappers.json"),
          "--format",
          "json",
          resolve(root, name),
        ],
        { encoding: "utf8" },
      );
      const expected = lint(
        fixture(`no-vitest-sequential/${name}`),
        {
          "no-mistakes/no-vitest-sequential": "error",
        },
        name,
      );
      assert.equal(result.status, expected.length ? 1 : 0, result.stderr);
      const actual = JSON.parse(result.stdout).diagnostics.filter((diagnostic) =>
        diagnostic.code.includes("no-vitest-sequential"),
      );
      assert.deepEqual(
        actual.map((diagnostic) => diagnostic.labels[0].span.line).sort((a, b) => a - b),
        expected.map((finding) => finding.line),
      );
    }
  });
});
