import assert from "node:assert/strict";
import { describe, expect, it } from "vitest";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, require } from "./helpers.mjs";

const { createOwnDataProperty } = require("../src/rules/vitest-timeout-copy-properties");
const { createCopySlotWrite } = require("../src/rules/vitest-timeout-copy-slots");

const rule = "vitest-timeout-cap";
const finding = { unknownValues: "finding" };
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function capCase(name, options = {}) {
  return {
    code: fixture(`vitest-timeout-cap/${name}.ts`),
    filename: "fixture.test.ts",
    languageOptions: { parser: tsParser },
    options: [options],
  };
}

const suite = {
  valid: [
    // Unknown accessor receivers, inherited descriptors, aliases, escapes and uncertain paths.
    capCase("copy-slot-unsafe-receivers"),
    // A receiver whose first const definition has no initializer.
    capCase("copy-slot-declared-receiver"),
  ],
  invalid: [
    // 10 timeout. Source budgets survive guarded replacement and deletion.
    capCase("copy-slot-replacement"),
    // 10 timeout, 12 unknown.
    capCase("copy-slot-replacement", finding),
    // 12 unknown.
    capCase("copy-slot-unsafe-receivers", finding),
    // 3 unknown.
    capCase("copy-slot-declared-receiver", finding),
  ],
};

describe("vitest-timeout-cap copy slot replacement", () => {
  it("requires a literal receiver and immutable acyclic bindings independently of value stability", () => {
    const linter = new Linter({ configType: "flat" });
    const observations = [];
    linter.verify(fixture("vitest-timeout-cap/copy-slot-receiver-proof.ts"), {
      plugins: {
        proof: {
          rules: {
            receiver: {
              create(context) {
                const check = createCopySlotWrite(context, createOwnDataProperty(context));
                return {
                  UnaryExpression(node) {
                    if (node.operator === "delete")
                      observations.push([
                        node.loc.start.line,
                        check(
                          node.argument,
                          node,
                          ["copy"],
                          [{ key: "copy", container: "object" }],
                        ),
                      ]);
                  },
                };
              },
            },
          },
        },
      },
      rules: { "proof/receiver": "error" },
    });
    assert.deepEqual(observations, [
      [2, true],
      ...[4, 7, 9, 11, 13, 16, 18].map((line) => [line, false]),
    ]);
  });

  it("snapshots vitest-timeout-copy-slots diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/vitest-timeout-copy-slots.snap");
  }, 30_000);
});
