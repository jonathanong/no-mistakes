import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { plugin } from "./helpers.mjs";

describe("scope compatibility", () => {
  it("reports setTimeout when scope.set is unavailable", () => {
    const reports = [];
    const listener = plugin.rules["playwright-no-set-timeout"].create({
      filename: "e2e.spec.ts",
      sourceCode: {
        getScope: () => ({
          set: undefined,
          variables: [{ name: "setTimeout", defs: [] }],
          upper: null,
        }),
      },
      report: (item) => reports.push(item),
    });

    listener.CallExpression({
      type: "CallExpression",
      callee: { type: "Identifier", name: "setTimeout" },
    });

    assert.equal(reports.length, 1);
    assert.equal(reports[0].messageId, "timeout");
  });

  it("ignores named test callbacks when fallback lookup cannot resolve a function", () => {
    const reports = [];
    const listener = plugin.rules["test-no-shared-state"].create({
      sourceCode: {
        getScope: () => ({
          set: undefined,
          variables: [
            {
              name: "sharedCallback",
              defs: [{ node: { type: "VariableDeclarator", init: { type: "Literal", value: 0 } } }],
              scope: { type: "module", block: { type: "Program" } },
            },
          ],
          upper: null,
        }),
      },
      report: (item) => reports.push(item),
    });

    const call = {
      type: "CallExpression",
      callee: { type: "Identifier", name: "test" },
      arguments: [
        { type: "Literal", value: "shared callback" },
        { type: "Identifier", name: "sharedCallback" },
      ],
    };

    listener.CallExpression(call);
    listener["CallExpression:exit"](call);
    listener["Program:exit"]();

    assert.equal(reports.length, 0);
  });
});
