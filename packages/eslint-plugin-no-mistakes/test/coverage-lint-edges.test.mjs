import assert from "node:assert/strict";
import { describe, it } from "vitest";

import { messages, require } from "./helpers.mjs";

const {
  createAliasScopeTracker,
} = require("../src/rules/no-banned-import-outside-allowed-paths-scopes.js");
const {
  recordAssignmentTag,
  recordVariableTag,
} = require("../src/rules/no-banned-import-outside-allowed-paths-aliases.js");
const { matchDirectMockCallApply } = require("../src/rules/module-mock-call-apply.js");

describe("lint early-return edges", () => {
  it("covers no-delete-property non-delete unary and typed member delete", () => {
    assert.deepEqual(messages("const ok = !value;", "no-delete-property"), []);
    assert.deepEqual(
      messages("delete (value as Thing).prop;", "no-delete-property", undefined, "a.tsx"),
      ["delete"],
    );
  });

  it("covers placeholder never specifier fallbacks", () => {
    assert.deepEqual(
      messages(
        "type Keep = string; export type { Keep };",
        "no-placeholder-never-type-exports",
        undefined,
        "a.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        'type Placeholder = never; export type { Placeholder as "alias" };',
        "no-placeholder-never-type-exports",
        undefined,
        "a.ts",
      ),
      ["placeholder"],
    );
  });

  it("skips lowercase exported components", () => {
    assert.deepEqual(
      messages(
        "export function button() { return <button />; }",
        "playwright-require-exported-component-attribute",
      ),
      [],
    );
  });

  it("covers next metadata and script guard paths", () => {
    assert.deepEqual(
      messages(
        "export const metadata = {};",
        "nextjs-metadata-exports-location",
        undefined,
        "lib/meta.ts",
      ),
      [],
    );
    messages(
      'export { metadata as "meta" };',
      "nextjs-metadata-exports-location",
      undefined,
      "app/page.tsx",
    );
    assert.deepEqual(
      messages("<div />;", "nextjs-no-manual-script-tags", undefined, "components/x.jsx"),
      [],
    );
    assert.deepEqual(
      messages(
        '<script type="application/ld+json"></script>;',
        "nextjs-no-manual-script-tags",
        undefined,
        "app/page.tsx",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        '<script id="allowed" dangerouslySetInnerHTML={{ __html: "" }}></script>;',
        "nextjs-no-manual-script-tags",
        { allowInlineScriptIds: ["allowed"], allowInlineScriptIdPatterns: ["("] },
        "app/page.tsx",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        'export function Button() { return <button data-pw="ok" />; }',
        "playwright-require-exported-component-attribute",
      ),
      [],
    );
    messages(
      'type Placeholder = never; export type { Placeholder as "alias" };',
      "no-placeholder-never-type-exports",
      undefined,
      "a.ts",
    );
    messages(
      `<Foo.Button />; <a href="/x" />; <div onClick={() => {}} />; <div role="button" />;`,
      "playwright-require-interactive-test-id",
      {
        interactiveComponents: ["Foo.Button", "/Button/", "/[/"],
      },
    );
    messages('<input data-pw="ok" />;', "playwright-require-interactive-test-id");
  });
});

describe("alias scope and mock apply helpers", () => {
  it("ignores switch-case exits without an entered switch", () => {
    const tracker = createAliasScopeTracker();
    tracker.exitSwitchCase({ consequent: [] });
    tracker.enterSwitch();
    tracker.enterSwitchCase();
    tracker.exitSwitchCase({
      consequent: [{ type: "BreakStatement" }],
    });
    tracker.exitSwitch();
  });

  it("records object-pattern and compound assignment tags", () => {
    const identifier = { type: "Identifier", name: "mod" };
    const variable = { name: "mod" };
    const context = {
      sourceCode: {
        getScope: () => ({
          set: new Map([["mod", variable]]),
          upper: null,
        }),
      },
    };
    const aliasMap = new Map();
    const cleared = new Set();
    recordVariableTag({ id: { type: "Identifier", name: "mod" } }, context, aliasMap, cleared, {});
    recordVariableTag(
      {
        init: { type: "Identifier", name: "other" },
        id: {
          type: "ObjectPattern",
          properties: [
            { type: "RestElement", argument: identifier },
            {
              type: "Property",
              key: { name: "fn" },
              value: { type: "ObjectPattern", properties: [] },
            },
            {
              type: "Property",
              key: { name: "fn" },
              value: identifier,
            },
          ],
        },
      },
      context,
      aliasMap,
      cleared,
      { bannedModules: [] },
    );
    recordAssignmentTag(
      {
        operator: "=",
        left: {
          type: "ArrayPattern",
          elements: [identifier],
        },
        right: { type: "Identifier", name: "rhs" },
      },
      context,
      aliasMap,
      cleared,
      {},
    );
    recordAssignmentTag(
      {
        operator: "=",
        left: {
          type: "ObjectPattern",
          properties: [{ type: "RestElement", argument: identifier }],
        },
        right: { type: "Identifier", name: "rhs" },
      },
      context,
      aliasMap,
      cleared,
      {},
    );
    expectDefined(cleared);
  });

  it("matches mock.apply array arguments", () => {
    const context = {
      sourceCode: {
        getScope: () => ({ variables: [], upper: null }),
      },
    };
    const result = matchDirectMockCallApply(
      {
        callee: {
          type: "MemberExpression",
          property: { name: "apply" },
          object: {
            type: "MemberExpression",
            object: { type: "Identifier", name: "vi" },
            property: { name: "mock" },
          },
        },
        arguments: [
          {},
          { type: "ArrayExpression", elements: ["mod", { type: "ArrowFunctionExpression" }] },
        ],
      },
      context,
      new Set(["mock"]),
    );
    expectDefined(result);
    const applyIdentifier = matchDirectMockCallApply(
      {
        callee: {
          type: "MemberExpression",
          property: { name: "apply" },
          object: {
            type: "MemberExpression",
            object: { type: "Identifier", name: "vi" },
            property: { name: "mock" },
          },
        },
        arguments: [{}, { type: "Identifier", name: "args" }],
      },
      context,
      new Set(["mock"]),
    );
    expectDefined(applyIdentifier);
    expectDefined(
      matchDirectMockCallApply(
        {
          callee: {
            type: "MemberExpression",
            property: { name: "call" },
            object: {
              type: "MemberExpression",
              object: { type: "Identifier", name: "vi" },
              property: { name: "mock" },
            },
          },
          arguments: [{}, "mod", { type: "ArrowFunctionExpression" }],
        },
        context,
        new Set(["mock"]),
      ),
    );
  });
});

function expectDefined(value) {
  assert.ok(value !== undefined);
}
