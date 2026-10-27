import assert from "node:assert/strict";
import { describe, expect, it } from "vitest";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, require } from "./helpers.mjs";

const { laterTopLevelDeletion } = require("../src/rules/vitest-timeout-descriptor-order");
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
    // Receivers without an initializer, including parsable repeated bindings.
    capCase("descriptor-delete-declared"),
    capCase("descriptor-delete-declared-copy"),
    // An earlier own assignment bypasses the setter until a prior delete exposes it.
    capCase("descriptor-delete-setter-exposure"),
  ],
  invalid: [
    // 2 timeout. A later delete does not retroactively invalidate an earlier direct assignment.
    capCase("descriptor-delete-order"),
    // 2 timeout, 36 unknown.
    capCase("descriptor-delete-order", finding),
    // 1 unknown.
    capCase("descriptor-delete-declared", finding),
    // 3 unknown.
    capCase("descriptor-delete-declared-copy", finding),
    // 3 unknown.
    capCase("descriptor-delete-setter-exposure", finding),
  ],
};

describe("vitest-timeout-cap descriptor deletion ordering", () => {
  it("requires direct operations and a literal identifier receiver independently of descriptor lookup", () => {
    const observations = [];
    new Linter({ configType: "flat" }).verify(
      fixture("vitest-timeout-cap/descriptor-delete-order-proof.ts"),
      {
        plugins: {
          proof: {
            rules: {
              order: {
                create(context) {
                  let member;
                  return {
                    MemberExpression(node) {
                      if (
                        ["AssignmentExpression", "ExpressionStatement"].includes(node.parent.type)
                      )
                        member = node;
                    },
                    UnaryExpression(node) {
                      if (node.operator === "delete")
                        observations.push([
                          node.loc.start.line,
                          laterTopLevelDeletion(member, node, context),
                        ]);
                    },
                  };
                },
              },
            },
          },
        },
        rules: { "proof/order": "error" },
      },
    );
    assert.deepEqual(observations, [
      [3, true],
      ...[6, 9, 11, 13, 16, 19, 22, 25, 27, 30].map((line) => [line, false]),
    ]);
  });

  it("snapshots vitest-timeout-descriptor-order diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/vitest-timeout-descriptor-order.snap");
  }, 30_000);

  it("verifies an earlier own assignment bypasses the setter and a prior delete exposes it", async () => {
    const { observed } =
      await import("../../../test-cases/eslint-snippets/fixture/vitest-timeout-cap/descriptor-delete-setter-exposure.ts");
    assert.deepEqual(observed, [0, 1, 60_000]);
  });
});
