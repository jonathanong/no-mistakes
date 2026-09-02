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

  it("does not report a beforeAll-local shadow of a describe-scope hoisted token", () => {
    const reports = [];

    // test.describe('D', () => {
    //   const suffix = randomSuffix();          // outer candidate, describe scope
    //   test.beforeAll(() => {
    //     const suffix = randomSuffix();        // inner shadow, already scoped to its own hook
    //     use(suffix);
    //   });
    // });
    const outerDeclarator = {
      type: "VariableDeclarator",
      id: { type: "Identifier", name: "suffix" },
      init: {
        type: "CallExpression",
        callee: { type: "Identifier", name: "randomSuffix" },
        arguments: [],
      },
    };
    const outerDeclaration = {
      type: "VariableDeclaration",
      kind: "const",
      declarations: [outerDeclarator],
    };
    outerDeclarator.parent = outerDeclaration;

    const innerDeclarator = {
      type: "VariableDeclarator",
      id: { type: "Identifier", name: "suffix" },
      init: {
        type: "CallExpression",
        callee: { type: "Identifier", name: "randomSuffix" },
        arguments: [],
      },
    };
    const innerDeclaration = {
      type: "VariableDeclaration",
      kind: "const",
      declarations: [innerDeclarator],
    };
    innerDeclarator.parent = innerDeclaration;

    const useStatement = {
      type: "ExpressionStatement",
      expression: {
        type: "CallExpression",
        callee: { type: "Identifier", name: "use" },
        arguments: [{ type: "Identifier", name: "suffix" }],
      },
    };

    const beforeAllCallback = {
      type: "ArrowFunctionExpression",
      params: [],
      body: { type: "BlockStatement", body: [innerDeclaration, useStatement] },
    };
    innerDeclaration.parent = beforeAllCallback.body;
    beforeAllCallback.body.parent = beforeAllCallback;

    const beforeAllCall = {
      type: "CallExpression",
      callee: {
        type: "MemberExpression",
        computed: false,
        object: { type: "Identifier", name: "test" },
        property: { type: "Identifier", name: "beforeAll" },
      },
      arguments: [beforeAllCallback],
    };
    beforeAllCallback.parent = beforeAllCall;

    const describeCallback = {
      type: "ArrowFunctionExpression",
      params: [],
      body: {
        type: "BlockStatement",
        body: [outerDeclaration, { type: "ExpressionStatement", expression: beforeAllCall }],
      },
    };
    outerDeclaration.parent = describeCallback.body;
    describeCallback.body.parent = describeCallback;

    const describeCall = {
      type: "CallExpression",
      callee: {
        type: "MemberExpression",
        computed: false,
        object: { type: "Identifier", name: "test" },
        property: { type: "Identifier", name: "describe" },
      },
      arguments: [{ type: "Literal", value: "D" }, describeCallback],
    };
    describeCallback.parent = describeCall;

    // Degraded scope shape: `scope.set` absent, resolution falls back to `scope.variables.find(...)`.
    // A single flat scope models the beforeAll callback's own block scope, where the inner shadow
    // resolves first — proving shadow-correctness comes from `resolveVariable`'s scope walk, not
    // from matching the declarator's name against every candidate.
    const listener = plugin.rules["playwright-no-hoisted-unique-token"].create({
      filename: "e2e.spec.ts",
      options: [{ tokenFactories: ["randomSuffix"] }],
      sourceCode: {
        getScope: () => ({
          set: undefined,
          variables: [{ name: "suffix", defs: [{ node: innerDeclarator }] }],
          upper: null,
        }),
      },
      report: (item) => reports.push(item),
    });

    listener.VariableDeclarator(outerDeclarator);
    listener.VariableDeclarator(innerDeclarator);
    listener.CallExpression(beforeAllCall);
    listener["Program:exit"]();

    assert.equal(reports.length, 0);
  });

  it("does not report a locally-declared scrollTo helper when scope.set is unavailable", () => {
    const reports = [];

    // A project's own `function scrollTo(...) {}` helper, called bare inside a Playwright file
    // that also has a qualifying cursor wait — must resolve to its own declaration, not the
    // global, under the degraded scope shape oxlint's partial scope implementation produces.
    const helperDeclarator = {
      type: "FunctionDeclaration",
      id: { type: "Identifier", name: "scrollTo" },
    };
    const waitCall = {
      type: "CallExpression",
      callee: {
        type: "MemberExpression",
        computed: false,
        object: { type: "Identifier", name: "page" },
        property: { type: "Identifier", name: "waitForRequest" },
      },
      arguments: [{ type: "Literal", value: "**/api/v1/posts?after=abc" }],
    };
    const scrollCall = {
      type: "CallExpression",
      callee: { type: "Identifier", name: "scrollTo" },
      arguments: [],
    };

    const listener = plugin.rules["playwright-no-raw-scroll-pagination"].create({
      filename: "e2e.spec.ts",
      sourceCode: {
        getScope: () => ({
          set: undefined,
          variables: [{ name: "scrollTo", defs: [{ node: helperDeclarator }] }],
          upper: null,
        }),
      },
      report: (item) => reports.push(item),
    });

    listener.CallExpression(waitCall);
    listener.CallExpression(scrollCall);
    listener["Program:exit"]();

    assert.equal(reports.length, 0);
  });
});
