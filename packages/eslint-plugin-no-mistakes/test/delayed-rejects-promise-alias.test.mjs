import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, fixture, lint } from "./helpers.mjs";

const rule = "test-no-delayed-rejects";
const filename = (name) => `${rule}/${name}.ts`;
function diagnostics(name) {
  return lint(fixture(filename(name)), { [`no-mistakes/${rule}`]: "error" }, filename(name));
}

const cases = [
  ["promise-alias-invalid", [7, 15, 22, 28, 34, 41, 48, 58, 66, 73]],
  ["promise-alias-controls", [66, 88]],
  ["promise-alias-suppressed", []],
  ["promise-alias-suppressed-file", []],
];

describe("delayed rejection promise-value aliases", () => {
  for (const [name, lines] of cases) {
    it(`keeps saved ${name} behavior`, () => {
      assert.deepEqual(
        diagnostics(name).map((diagnostic) => [diagnostic.line, diagnostic.messageId]),
        lines.map((line) => [line, "delayedReject"]),
      );
    });
  }

  it("matches Oxlint on every saved alias case", () => {
    const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture");
    for (const [name, lines] of cases) {
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, rule, ".oxlintrc-forwarding.json"),
          "--format",
          "json",
          resolve(root, filename(name)),
        ],
        { encoding: "utf8" },
      );
      assert.equal(result.status, lines.length ? 1 : 0, `${name}: ${result.stderr}`);
      const actual = JSON.parse(result.stdout).diagnostics.filter(
        (diagnostic) => diagnostic.code === `no-mistakes(${rule})`,
      );
      assert.deepEqual(
        actual.map((diagnostic) => diagnostic.labels[0].span.line),
        lines,
        name,
      );
    }
  });
});
