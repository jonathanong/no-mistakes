import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, messages, require } from "./helpers.mjs";

const snippet = (name) => fixture(name);

describe("test and react same-file recall", () => {
  it("reports vitest sequential through aliases and static keys", () => {
    assert.equal(
      messages(
        snippet("no-vitest-sequential/recall.test.ts"),
        "no-vitest-sequential",
        undefined,
        "recall.test.ts",
      ).length,
      14,
    );
  });

  it("reports immediately invoked functions reached through forwarding", () => {
    assert.deepEqual(messages(snippet("react-no-iife-in-jsx/recall.tsx"), "react-no-iife-in-jsx"), [
      "iife",
      "iife",
      "iife",
      "iife",
    ]);
  });

  it("reports nullish react nodes reached through static keys and const aliases", () => {
    assert.equal(
      messages(
        snippet("react-no-nullish-react-node/recall.tsx"),
        "react-no-nullish-react-node",
        undefined,
        "recall.tsx",
      ).length,
      4,
    );
  });

  it("reports React.use(Promise.resolve) through the same-file spellings", () => {
    assert.equal(
      messages(
        snippet("react-no-use-promise-resolve/recall.tsx"),
        "react-no-use-promise-resolve",
        undefined,
        "recall.tsx",
      ).length,
      10,
    );
  });

  it("treats an expression-free template require as a test import", () => {
    assert.deepEqual(
      messages(
        snippet("no-import-only-test-files/recall.test.js"),
        "no-import-only-test-files",
        undefined,
        "recall.test.js",
      ),
      ["aggregate"],
    );
    assert.deepEqual(
      messages(
        snippet("no-import-only-test-files/dynamic.test.js"),
        "no-import-only-test-files",
        undefined,
        "dynamic.test.js",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        snippet("no-import-only-test-files/suppressed.test.js"),
        "no-import-only-test-files",
        undefined,
        "suppressed.test.js",
      ),
      [],
    );
  });

  it("recognizes vitest namespace module mocks and ignores the default export", () => {
    assert.deepEqual(
      messages(
        snippet("vitest-mock-test-file-naming/recall.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "recall.test.ts",
      ),
      ["needsMock"],
    );
    assert.deepEqual(
      messages(
        snippet("vitest-mock-test-file-naming/default.mock.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "default.mock.test.ts",
      ),
      ["unnecessaryMock"],
    );
  });

  it("reports error message assertions through expect aliases and static keys", () => {
    assert.equal(
      messages(
        snippet("test-no-error-message-matching/recall.test.ts"),
        "test-no-error-message-matching",
        undefined,
        "recall.test.ts",
      ).length,
      14,
    );
  });

  it("reports shared state inside aliased and namespace test calls", () => {
    assert.equal(
      messages(
        snippet("test-no-shared-state/recall.test.ts"),
        "test-no-shared-state",
        undefined,
        "recall.test.ts",
      ).length,
      5,
    );
    assert.equal(
      messages(
        snippet("test-no-shared-state/computed-describe.test.ts"),
        "test-no-shared-state",
        undefined,
        "computed-describe.test.ts",
      ).length,
      1,
    );
    assert.equal(
      messages(
        snippet("test-no-shared-state/alias-describe.test.ts"),
        "test-no-shared-state",
        undefined,
        "alias-describe.test.ts",
      ).length,
      1,
    );
    assert.equal(
      messages(
        snippet("test-no-shared-state/sequence-describe.test.ts"),
        "test-no-shared-state",
        undefined,
        "sequence-describe.test.ts",
      ).length,
      1,
    );
  });

  it("reports delayed rejects through renamed expect and static keys", () => {
    assert.equal(
      messages(
        snippet("test-no-delayed-rejects/recall.ts"),
        "test-no-delayed-rejects",
        undefined,
        "recall.ts",
      ).length,
      3,
    );
  });

  it("covers static-key and missing-binding helper fallbacks", () => {
    const sequential = require("../src/rules/no-vitest-sequential.js").__test;
    const expectHelpers = require("../src/rules/test-message-expect.js");
    const mocks = require("../src/rules/vitest-mock-test-file-naming.js").__test;
    const context = { sourceCode: { getScope: () => null } };
    assert.equal(sequential.staticName(null, false), null);
    assert.equal(sequential.staticName({ type: "Identifier", name: "call" }, true), null);
    assert.equal(sequential.staticName({ name: "sequential" }, false), "sequential");
    assert.equal(
      sequential.staticName({ type: "Literal", value: "sequential" }, true),
      "sequential",
    );
    assert.equal(sequential.destructuredTest(null, context, new Set()), false);
    assert.equal(
      sequential.destructuredTest({ type: "Identifier", name: "check" }, context, new Set(["v"])),
      false,
    );
    assert.equal(sequential.hasSequentialMember(null), false);
    assert.equal(
      sequential.constInit(
        { type: "Identifier", name: "check" },
        {
          sourceCode: {
            getScope: () => ({
              upper: null,
              variables: [
                {
                  name: "check",
                  defs: [
                    {
                      type: "Variable",
                      node: {
                        parent: { kind: "const" },
                        init: { type: "Identifier", name: "v" },
                        id: { type: "ObjectPattern", properties: [] },
                      },
                    },
                  ],
                },
              ],
            }),
          },
        },
        new Set(),
      ),
      null,
    );
    assert.equal(
      sequential.hasSequentialOption({
        arguments: [
          { type: "Literal", value: "name" },
          {
            type: "ObjectExpression",
            properties: [
              { type: "RestElement" },
              {
                type: "Property",
                computed: true,
                key: {
                  type: "TemplateLiteral",
                  expressions: [],
                  quasis: [{ value: { cooked: "sequential" } }],
                },
                value: { type: "Literal", value: true },
              },
            ],
          },
        ],
      }),
      true,
    );
    assert.equal(
      expectHelpers.isExpectCallee(
        { type: "Identifier", name: "check" },
        {
          sourceCode: {
            getScope: () => ({
              upper: null,
              variables: [
                {
                  name: "check",
                  defs: [
                    {
                      type: "Variable",
                      node: {
                        parent: { kind: "const" },
                        init: { type: "Identifier", name: "v" },
                        id: { type: "ArrayPattern" },
                      },
                    },
                  ],
                },
              ],
            }),
          },
        },
      ),
      false,
    );
    assert.equal(expectHelpers.memberName(null), null);
    assert.equal(expectHelpers.propertyName(null), null);
    assert.equal(expectHelpers.propertyName({ type: "Literal", value: 1 }), "1");
    assert.equal(expectHelpers.isExpectCall(null, context), false);
    assert.equal(expectHelpers.expectArguments({ type: "Identifier" }, context), null);
    assert.equal(mocks.propertyName(null, false), null);
    assert.equal(mocks.propertyName({ type: "Identifier", name: "mock" }, true), null);
    assert.equal(mocks.mockMember(null, context), false);
    assert.equal(mocks.mockMember({ type: "Identifier", name: "mock" }, context), false);
    const bound = {
      sourceCode: {
        getScope: () => ({
          upper: null,
          variables: [
            {
              name: "check",
              defs: [
                {
                  type: "Variable",
                  node: {
                    parent: { kind: "const" },
                    init: { type: "Identifier", name: "v" },
                    id: {
                      type: "ObjectPattern",
                      properties: [
                        { type: "RestElement" },
                        {
                          type: "Property",
                          computed: false,
                          key: { type: "Identifier", name: "it" },
                          value: {
                            type: "AssignmentPattern",
                            left: { type: "Identifier", name: "check" },
                          },
                        },
                      ],
                    },
                  },
                },
              ],
            },
          ],
        }),
      },
    };
    assert.equal(
      sequential.destructuredTest({ type: "Identifier", name: "check" }, bound, new Set()),
      false,
    );
    assert.equal(
      sequential.destructuredTest({ type: "Identifier", name: "check" }, bound, new Set(["v"])),
      true,
    );
    const member = (name, object, computed = false) => ({
      type: "MemberExpression",
      computed,
      property: { type: "Identifier", name },
      object,
    });
    assert.equal(
      expectHelpers.isExpectCallee(member("call", { type: "Identifier", name: "expect" }), context),
      true,
    );
    assert.equal(
      expectHelpers.isExpectCallee(member("soft", { type: "Identifier", name: "expect" }), context),
      true,
    );
    assert.equal(
      expectHelpers.isExpectCallee(
        member("expect", { type: "Identifier", name: "v" }, true),
        context,
      ),
      false,
    );
    assert.equal(
      expectHelpers.isNamespace({ type: "Identifier", name: "v" }, context, new Set()),
      false,
    );
    assert.equal(
      expectHelpers.isNamespace({ type: "Identifier", name: "v" }, context, new Set(["v"])),
      false,
    );
    assert.deepEqual(
      expectHelpers.expectArguments(
        {
          type: "CallExpression",
          callee: member("apply", { type: "Identifier", name: "expect" }),
          arguments: [
            { type: "Literal", value: null },
            { type: "ArrayExpression", elements: [] },
          ],
        },
        context,
      ),
      [],
    );
    assert.equal(
      mocks.mockMember(
        member("mock", { type: "CallExpression", callee: { type: "Identifier", name: "getVi" } }),
        context,
      ),
      false,
    );
    const reactUse = require("../src/rules/react-no-use-promise-resolve.js").__test;
    const iife = require("../src/rules/react-no-iife-in-jsx.js").__test;
    assert.equal(reactUse.staticName(null, false), null);
    assert.equal(reactUse.staticName({ type: "Identifier", name: "use" }, true), null);
    assert.equal(reactUse.patternKey(null, "use"), null);
    assert.equal(
      reactUse.patternKey(
        {
          type: "ObjectPattern",
          properties: [
            { type: "RestElement" },
            {
              type: "Property",
              computed: true,
              key: {
                type: "TemplateLiteral",
                expressions: [],
                quasis: [{ value: { cooked: "use" } }],
              },
              value: { type: "AssignmentPattern", left: { type: "Identifier", name: "taken" } },
            },
          ],
        },
        "taken",
      ),
      "use",
    );
    assert.equal(reactUse.isReactUse(null, context, new Set()), false);
    assert.equal(reactUse.isPromiseResolve(null), false);
    assert.equal(reactUse.isPromiseResolve({ type: "Identifier", name: "Promise" }), false);
    assert.equal(reactUse.isReactNamespace(null, context), false);
    assert.equal(iife.isFunctionCallee(null), false);
    assert.equal(
      iife.isFunctionCallee(member("call", { type: "FunctionExpression" }, true)),
      false,
    );
    assert.equal(
      iife.isFunctionCallee({
        type: "MemberExpression",
        computed: false,
        property: { type: "Literal", value: "call" },
        object: { type: "ArrowFunctionExpression" },
      }),
      true,
    );
    assert.equal(iife.isIife({ type: "Identifier" }), false);
    const cycle = {
      sourceCode: {
        getScope: () => ({
          upper: null,
          variables: [
            {
              name: "use",
              defs: [
                {
                  type: "Variable",
                  node: {
                    parent: { kind: "const" },
                    id: { type: "Identifier" },
                    init: { type: "Identifier", name: "use" },
                  },
                },
              ],
            },
          ],
        }),
      },
    };
    assert.equal(reactUse.isReactUse({ type: "Identifier", name: "use" }, cycle, new Set()), false);
    assert.equal(
      mocks.mockMember(
        { type: "Identifier", name: "install" },
        {
          sourceCode: {
            getScope: () => ({
              upper: null,
              variables: [
                {
                  name: "install",
                  defs: [
                    {
                      type: "Variable",
                      node: {
                        parent: { kind: "const" },
                        init: { type: "Identifier", name: "vi" },
                        id: {
                          type: "ObjectPattern",
                          properties: [
                            { type: "RestElement" },
                            {
                              type: "Property",
                              computed: true,
                              key: {
                                type: "TemplateLiteral",
                                expressions: [],
                                quasis: [{ value: { cooked: "mock" } }],
                              },
                              value: {
                                type: "AssignmentPattern",
                                left: { type: "Identifier", name: "install" },
                              },
                            },
                          ],
                        },
                      },
                    },
                  ],
                },
              ],
            }),
          },
        },
      ),
      true,
    );
    const imports = require("../src/rules/no-import-only-test-files.js").__test;
    assert.equal(imports.requireSource(null), null);
    assert.equal(
      imports.requireSource({
        type: "CallExpression",
        callee: {
          type: "ChainExpression",
          expression: { type: "Identifier", name: "require" },
        },
        arguments: [{ type: "Literal", value: "./a.test" }],
      }),
      "./a.test",
    );
    assert.equal(
      imports.requireSource({
        type: "CallExpression",
        callee: {
          type: "SequenceExpression",
          expressions: [
            { type: "Literal", value: 0 },
            { type: "Identifier", name: "require" },
          ],
        },
        arguments: [
          { type: "TemplateLiteral", expressions: [], quasis: [{ value: { cooked: "./b.test" } }] },
        ],
      }),
      "./b.test",
    );
    assert.equal(
      imports.requireSource({
        type: "CallExpression",
        callee: member("apply", { type: "Identifier", name: "require" }),
        arguments: [
          { type: "Literal", value: null },
          { type: "Identifier", name: "args" },
        ],
      }),
      null,
    );
    assert.equal(
      imports.requireSource({
        type: "CallExpression",
        callee: { type: "Identifier", name: "require" },
        arguments: [{ type: "Literal", value: 1 }],
      }),
      null,
    );
  });
});
