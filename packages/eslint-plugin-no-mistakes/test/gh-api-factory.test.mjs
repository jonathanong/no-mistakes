import assert from "node:assert/strict";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { fixture, lint, __dirname } from "./helpers.mjs";

const id = "gh-api-argument-safety";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
const factoryWrappers = [
  { factory: "makeClient", method: "call", argumentPosition: 1, includesApi: false },
];
function check(options) {
  return lint(
    fixture(`${id}/factory.ts`),
    { [`no-mistakes/${id}`]: ["error", options] },
    resolve(root, "factory.ts"),
  );
}
describe("gh-api-argument-safety factoryWrappers", () => {
  it("selects only the configured factory receiver and method", async () => {
    const messages = check({ factoryWrappers });
    assert.deepEqual(
      messages.map((m) => [m.line, m.messageId]),
      [
        [4, "endpointQuery"],
        [5, "missingMethod"],
        [9, "endpointQuery"],
        [15, "endpointQuery"],
      ],
    );
    const rows = messages.map(({ line, column, messageId, message }) => ({
      file: "factory.ts",
      line,
      column,
      messageId,
      message,
    }));
    await expect(`${JSON.stringify(rows, null, 2)}\n`).toMatchFileSnapshot(
      "snapshots/gh-api-factory.snap",
    );
  });
  it("treats the first argv element as api when includesApi is not false", () => {
    const messages = check({
      factoryWrappers: [{ factory: "makeClient", method: "call", argumentPosition: 1 }],
    });
    assert.deepEqual(
      messages.map((m) => m.line),
      [11],
    );
  });
  it("stays silent when the option is not configured", () => {
    assert.deepEqual(check({}), []);
    assert.deepEqual(check({ factoryWrappers: [] }), []);
  });
});
