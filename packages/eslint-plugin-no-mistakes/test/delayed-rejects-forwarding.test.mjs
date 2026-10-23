import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { fixture, lint, __dirname } from "./helpers.mjs";

const id = "test-no-delayed-rejects";
function check(name) {
  return lint(fixture(`${id}/${name}.ts`), { [`no-mistakes/${id}`]: "error" }, `${name}.ts`);
}
describe("delayed rejection forwarding", () => {
  it("reads the promise argument after call/apply forwarding", () => {
    assert.deepEqual(
      check("forwarding").map((message) => [message.line, message.messageId]),
      [
        [9, "delayedReject"],
        [14, "delayedReject"],
        [19, "delayedReject"],
        [24, "delayedReject"],
        [30, "delayedReject"],
      ],
    );
  });
  it("preserves malformed calls, poll, unknown arguments, shadowing and early observers", () => {
    assert.deepEqual(check("forwarding-controls"), []);
  });
  it("honors file, line and next-line suppression", () => {
    assert.deepEqual(check("forwarding-suppressed"), []);
    assert.deepEqual(check("forwarding-suppressed-file"), []);
  });
  it("shares proven nested aliases with error-message assertions, preserving shadowing", () => {
    const rule = "test-no-error-message-matching";
    assert.deepEqual(
      lint(
        fixture(`${rule}/nested-alias.ts`),
        {
          [`no-mistakes/${rule}`]: "error",
        },
        "nested-alias.ts",
      ).map((message) => [message.line, message.messageId]),
      [
        [4, "message"],
        [6, "message"],
      ],
    );
  });
  it("matches Oxlint for forwarded observers, controls and suppressions", () => {
    const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture");
    for (const [file, count] of [
      [`${id}/forwarding.ts`, 5],
      [`${id}/forwarding-controls.ts`, 0],
      [`${id}/forwarding-suppressed.ts`, 0],
      [`${id}/forwarding-suppressed-file.ts`, 0],
      ["test-no-error-message-matching/nested-alias.ts", 2],
    ]) {
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, id, ".oxlintrc-forwarding.json"),
          "--format",
          "json",
          resolve(root, file),
        ],
        { encoding: "utf8" },
      );
      assert.equal(result.status, count ? 1 : 0, `${file}: ${result.stderr}`);
      assert.equal(JSON.parse(result.stdout).diagnostics.length, count, file);
    }
  });
});
