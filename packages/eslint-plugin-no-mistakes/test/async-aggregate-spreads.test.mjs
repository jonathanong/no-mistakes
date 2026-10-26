import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { it } from "vitest";
import { __dirname, lint } from "./helpers.mjs";

const root = resolve(
  __dirname,
  "../../../test-cases/eslint-plugin/async-call-disposition/fixture/aggregate-spreads",
);
const caller = resolve(root, "caller.ts");
const expected = [
  11, 12, 13, 14, 15, 16, 17, 23, 24, 25, 26, 27, 28, 29, 35, 36, 37, 38, 39, 40, 41, 47, 48, 49,
  50, 51, 52, 53, 54, 55, 56, 57,
];
const rules = {
  "no-mistakes/async-call-disposition": [
    "error",
    {
      targets: [{ sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["enqueueEmail"] }],
    },
  ],
};

// Run the production adapter first: ESLint AST parity alone misses Oxlint regressions.
it.each([false, true])(
  "retains aggregate controls in real type-aware Oxlint (masked=%s)",
  (masked) => {
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--type-aware",
        "--deny-warnings",
        "--config",
        resolve(root, ".oxlintrc.json"),
        "--format",
        "json",
        caller,
      ],
      {
        encoding: "utf8",
        env: {
          ...process.env,
          NODE_OPTIONS: masked
            ? `--require=${JSON.stringify(resolve(root, "mask-optional-peers.cjs"))}`
            : "",
        },
      },
    );
    assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
    assert.deepEqual(
      JSON.parse(result.stdout).diagnostics.map((item) => [item.labels[0].span.line, item.code]),
      expected.map((line) => [line, "no-mistakes(async-call-disposition)"]),
    );
  },
);

it("accepts conditional spread members while preserving dropped and nested controls in ESLint", () => {
  assert.deepEqual(
    lint(readFileSync(caller, "utf8"), rules, caller).map(({ line, messageId }) => [
      line,
      messageId,
    ]),
    expected.map((line) => [line, "disposition"]),
  );
});
