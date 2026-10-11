import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { Linter } from "eslint";
import { fixture, lint, require } from "./helpers.mjs";
const { laterTopLevelDeletion } = require("../src/rules/vitest-timeout-descriptor-order");
const findings = (name, options = {}) =>
  lint(
    fixture(`vitest-timeout-cap/${name}.ts`),
    { "no-mistakes/vitest-timeout-cap": ["error", options] },
    "fixture.test.ts",
  ).map(({ line, messageId }) => [line, messageId]);

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
  it("does not retroactively invalidate earlier direct assignments but retains uncertain order", () => {
    assert.deepEqual(findings("descriptor-delete-order"), [
      [9, "timeout"],
      [95, "timeout"],
    ]);
    assert.deepEqual(findings("descriptor-delete-order", { unknownValues: "finding" }), [
      [9, "timeout"],
      ...[15, 21, 28, 35, 40, 46, 53, 60, 66, 74, 81, 88].flatMap((line) =>
        Array(3).fill([line, "unknown"]),
      ),
      [95, "timeout"],
    ]);
  });

  it("keeps receivers without an initializer unknown, including parsable repeated bindings", () => {
    assert.deepEqual(findings("descriptor-delete-declared"), []);
    assert.deepEqual(findings("descriptor-delete-declared", { unknownValues: "finding" }), [
      [6, "unknown"],
    ]);
    assert.deepEqual(findings("descriptor-delete-declared-copy"), []);
    assert.deepEqual(
      findings("descriptor-delete-declared-copy", { unknownValues: "finding" }),
      Array(3).fill([6, "unknown"]),
    );
  });

  it("verifies an earlier own assignment bypasses the setter and a prior delete exposes it", async () => {
    assert.deepEqual(findings("descriptor-delete-setter-exposure"), []);
    assert.deepEqual(
      findings("descriptor-delete-setter-exposure", { unknownValues: "finding" }),
      Array(3).fill([12, "unknown"]),
    );
    const { observed } =
      await import("../../../test-cases/eslint-snippets/fixture/vitest-timeout-cap/descriptor-delete-setter-exposure.ts");
    assert.deepEqual(observed, [0, 1, 60_000]);
  });
});
