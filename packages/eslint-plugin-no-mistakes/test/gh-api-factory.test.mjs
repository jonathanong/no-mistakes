import assert from "node:assert/strict";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { fixture, lint, __dirname } from "./helpers.mjs";

const id = "gh-api-argument-safety";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
const factoryWrappers = [
  { factory: "makeClient", method: "call", argumentPosition: 1, includesApi: false },
];
function check(options, name = "factory.ts") {
  return lint(
    fixture(`${id}/${name}`),
    { [`no-mistakes/${id}`]: ["error", options] },
    resolve(root, name),
  );
}
function rows(messages, file) {
  return messages.map(({ line, column, messageId, message }) => ({
    file,
    line,
    column,
    messageId,
    message,
  }));
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
    await expect(`${JSON.stringify(rows(messages, "factory.ts"), null, 2)}\n`).toMatchFileSnapshot(
      "snapshots/gh-api-factory.snap",
    );
  });
  it("keeps optional calls, awaited receivers and spreads conservative", async () => {
    const messages = check({ factoryWrappers }, "factory-edges.ts");
    assert.deepEqual(
      messages.map((m) => [m.line, m.messageId]),
      [
        [4, "endpointQuery"],
        [5, "endpointQuery"],
        [8, "endpointQuery"],
      ],
    );
    await expect(
      `${JSON.stringify(rows(messages, "factory-edges.ts"), null, 2)}\n`,
    ).toMatchFileSnapshot("snapshots/gh-api-factory-edges.snap");
  });
  it("reports only the argv at the configured argumentPosition", async () => {
    const past = check(
      {
        factoryWrappers: [
          { factory: "makeClient", method: "call", argumentPosition: 2, includesApi: false },
        ],
      },
      "factory-edges.ts",
    );
    assert.deepEqual(
      past.map((m) => [m.line, m.messageId]),
      [[10, "endpointQuery"]],
    );
    await expect(
      `${JSON.stringify(rows(past, "factory-edges.ts"), null, 2)}\n`,
    ).toMatchFileSnapshot("snapshots/gh-api-factory-edges-position.snap");
  });
  it("does not inspect an argumentPosition past the end of the call", async () => {
    const past = check(
      {
        factoryWrappers: [
          { factory: "makeClient", method: "call", argumentPosition: 2, includesApi: false },
        ],
      },
      "factory-past.ts",
    );
    assert.deepEqual(past, []);
    const present = check(
      {
        factoryWrappers: [
          { factory: "makeClient", method: "call", argumentPosition: 1, includesApi: false },
        ],
      },
      "factory-past.ts",
    );
    assert.deepEqual(
      present.map((message) => [message.line, message.messageId]),
      [[3, "endpointQuery"]],
    );
    await expect(
      `${JSON.stringify(rows(present, "factory-past.ts"), null, 2)}\n`,
    ).toMatchFileSnapshot("snapshots/gh-api-factory-past.snap");
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
