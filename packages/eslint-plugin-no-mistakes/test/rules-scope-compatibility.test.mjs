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

  it("reports shared module-state writes when scope.set.get is unavailable", () => {
    const reports = [];
    const scope = {
      set: {},
      variables: [
        {
          name: "sharedState",
          defs: [{ node: { type: "VariableDeclarator", init: { type: "ArrayExpression" } } }],
          scope: { type: "module", block: { type: "Program" } },
        },
      ],
      upper: null,
    };
    const sourceCode = {
      getScope: () => scope,
    };

    const listener = plugin.rules["test-no-shared-state"].create({
      filename: "e2e.spec.ts",
      sourceCode,
      report: (item) => reports.push(item),
    });

    listener["Program > VariableDeclaration"]({
      type: "VariableDeclaration",
      kind: "const",
      declarations: [
        { id: { type: "Identifier", name: "sharedState" }, init: { type: "ArrayExpression" } },
      ],
    });

    const testCall = {
      type: "CallExpression",
      callee: { type: "Identifier", name: "test" },
      arguments: [{ type: "Literal", value: "shared state" }, { type: "ArrowFunctionExpression" }],
    };
    const assignment = {
      type: "AssignmentExpression",
      left: { type: "Identifier", name: "sharedState" },
      right: { type: "Literal", value: 1 },
      operator: "=",
    };

    listener.CallExpression(testCall);
    listener.AssignmentExpression(assignment);
    listener["CallExpression:exit"](testCall);

    assert.equal(reports.length, 1);
    assert.equal(reports[0].messageId, "shared");
  });

  it("treats shadowed fetch as shadowed when scope.set.get is unavailable", () => {
    const reports = [];
    const listener = plugin.rules["nextjs-static-fetch-url"].create({
      sourceCode: {
        getScope: () => ({
          set: {},
          variables: [
            {
              name: "fetch",
              defs: [{ type: "ImportBinding" }],
            },
          ],
          upper: null,
        }),
      },
      report: (item) => reports.push(item),
    });

    listener.CallExpression({
      type: "CallExpression",
      callee: { type: "Identifier", name: "fetch" },
      arguments: [{ type: "Identifier", name: "url" }],
    });

    assert.equal(reports.length, 0);
  });
});
