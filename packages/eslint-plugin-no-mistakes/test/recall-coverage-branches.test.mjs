import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { plugin, require } from "./helpers.mjs";

const expectApi = require("../src/rules/test-message-expect.js");
const aliases = require("../src/rules/test-no-shared-state-aliases.js");
const { createBindings } = require("../src/rules/playwright-test-timeout-bindings.js");
const sequential = plugin.rules["no-vitest-sequential"];

function ident(name) {
  return { type: "Identifier", name };
}

function ctx(variables) {
  return { sourceCode: { getScope: () => ({ variables, upper: null }) } };
}

function variable(name, defs) {
  return { name, defs };
}

describe("recall coverage branches", () => {
  it("resolves expect through literal imports, shadows, and destructures", () => {
    assert.equal(expectApi.propertyName({ name: "expect" }), "expect");
    assert.equal(
      expectApi.propertyName({ name: "expect", type: "Literal", value: "other" }),
      "other",
    );
    const context = ctx([
      variable("twice", [{ type: "Variable" }, { type: "Variable" }]),
      variable("renamed", [
        {
          type: "ImportBinding",
          node: { imported: { type: "Literal", value: "expect" } },
          parent: { source: { value: "vitest" } },
        },
      ]),
      variable("local", [
        {
          type: "Variable",
          node: {
            parent: { kind: "const" },
            id: {
              type: "ObjectPattern",
              properties: [
                {
                  type: "Property",
                  computed: true,
                  key: { type: "Literal", value: "expect" },
                  value: ident("local"),
                },
              ],
            },
            init: call(ident("require"), [{ type: "Literal", value: "vitest" }]),
          },
        },
      ]),
    ]);
    assert.equal(expectApi.isExpectCallee({ type: "Literal" }, context), false);
    assert.equal(expectApi.isExpectCallee(ident("expect"), context, new Set(["expect"])), false);
    assert.equal(expectApi.isExpectCallee(ident("twice"), context), false);
    assert.equal(
      expectApi.isExpectCallee(ident("renamed"), context, new Set(), new Set(["vitest"])),
      true,
    );
    assert.equal(expectApi.isExpectCallee(ident("local"), context), true);
    assert.equal(
      expectApi.isExpectCallee(
        ident("local"),
        ctx([
          variable("local", [
            {
              type: "Variable",
              node: {
                parent: { kind: "const" },
                id: { type: "ObjectPattern", properties: [] },
                init: ident("missing"),
              },
            },
          ]),
        ]),
      ),
      false,
    );
  });

  it("follows describe through a chain and a comma expression", () => {
    const chain = { type: "ChainExpression", expression: ident("describe") };
    assert.equal(aliases.hasProperty(chain, "describe"), false);
    const sequenced = {
      type: "SequenceExpression",
      expressions: [ident("test"), ident("describe")],
    };
    const testAliases = {
      isDescribeAliasCall: () => false,
      hasProperty: () => false,
    };
    assert.equal(
      aliases.isDescribeBinding(
        sequenced,
        testAliases,
        new Set(),
        new Set(),
        aliases.hasProperty,
        () => false,
      ),
      true,
    );
    assert.equal(
      aliases.isDescribeBinding(
        { type: "SequenceExpression", expressions: [] },
        testAliases,
        new Set(),
        new Set(),
        aliases.hasProperty,
        () => false,
      ),
      false,
    );
  });

  it("rejects a renamed runner imported from another module", () => {
    const check = variable("check", [
      {
        type: "ImportBinding",
        node: {
          type: "ImportSpecifier",
          imported: ident("test"),
        },
        parent: { source: { value: "node:test" } },
      },
      {
        type: "Variable",
        node: { parent: { kind: "const" }, id: ident("check"), init: ident("test") },
      },
    ]);
    const visitors = sequential.create({
      filename: "example.test.ts",
      options: [],
      report() {},
      sourceCode: {
        getScope: () => ({ variables: [check], upper: null }),
        getAllComments: () => [],
        ast: { tokens: [{ range: [1, 2] }] },
      },
    });
    visitors.CallExpression({
      callee: {
        type: "MemberExpression",
        computed: false,
        object: ident("check"),
        property: ident("sequential"),
      },
      arguments: [],
      loc: { start: { line: 1 } },
    });
    assert.equal(typeof visitors.CallExpression, "function");
  });

  it("resolves Playwright timeout bindings beyond a plain test name", () => {
    const reference = (isWrite, init) => ({ isWrite: () => isWrite, init });
    const pw = variable("pw", [
      {
        type: "ImportBinding",
        node: { type: "ImportNamespaceSpecifier" },
        parent: { source: { value: "@playwright/test" } },
      },
    ]);
    const test = variable("test", [
      {
        type: "ImportBinding",
        node: { type: "ImportSpecifier", imported: { name: "test" } },
        parent: { source: { value: "@playwright/test" } },
      },
    ]);
    const helper = variable("helper", [
      {
        type: "ImportBinding",
        node: { type: "ImportSpecifier", imported: { name: "helper" } },
        parent: { source: { value: "./extra" } },
      },
    ]);
    const mystery = variable("mystery", [
      {
        type: "ImportBinding",
        node: { type: "ImportSpecifier", imported: { name: "mystery" } },
        parent: { source: { value: "./extra" } },
      },
    ]);
    const run = variable("run", [{ type: "FunctionName", node: { type: "FunctionDeclaration" } }]);
    run.references = [reference(true, false)];
    const api = createBindings(
      ctx([pw, test, helper, mystery, run]),
      new Set(["@playwright/test", "./extra"]),
      { "./extra": { helper: "assertion", mystery: "opaque" } },
      [],
    );
    const member = (object, name) => ({
      type: "MemberExpression",
      computed: false,
      object,
      property: ident(name),
    });
    assert.equal(api.resolve(member(ident("pw"), "defineConfig")).kind, "defineConfig");
    assert.equal(api.resolve(ident("helper")), null);
    assert.equal(api.resolve(ident("mystery")).kind, "opaque");
    assert.equal(api.resolve(null), null);
    assert.equal(api.resolve(call(member(ident("pw"), "bind"), [ident("pw")])), null);
    assert.equal(
      api.resolve(call(member(member(ident("test"), "describe"), "bind"), [ident("other")])).kind,
      "partial",
    );
    assert.equal(api.resolve(call(ident("mystery"))).kind, "opaque");
    assert.equal(
      api.resolve(member(call(member(ident("test"), "info")), "setTimeout")).kind,
      "method",
    );
    const extra = variable("extra", [
      {
        type: "ImportBinding",
        node: { type: "ImportNamespaceSpecifier" },
        parent: { source: { value: "./extra" } },
      },
    ]);
    const info = variable("info", [{ type: "Parameter", node: ident("info") }]);
    const loop = variable("loop", [
      {
        type: "Variable",
        parent: { kind: "const" },
        node: { id: ident("loop"), init: ident("loop") },
      },
    ]);
    loop.references = [];
    const more = createBindings(
      ctx([pw, extra, info, loop]),
      new Set(["./extra"]),
      { "./extra": { mystery: "opaque" } },
      [],
    );
    assert.equal(more.resolve(member(ident("extra"), "mystery")).kind, "opaque");
    assert.equal(more.resolve(ident("info")), null);
    assert.equal(more.opaqueCallback(ident("loop")), false);
    assert.equal(
      more.resolve({
        type: "MemberExpression",
        computed: true,
        object: ident("extra"),
        property: {
          type: "BinaryExpression",
          operator: "+",
          left: { type: "Literal", value: null },
          right: { type: "Literal", value: "mystery" },
        },
      }).kind,
      "opaque",
    );
  });
});

function call(callee, args) {
  return { type: "CallExpression", callee, arguments: args };
}
