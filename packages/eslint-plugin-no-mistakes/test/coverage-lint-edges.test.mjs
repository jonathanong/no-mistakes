import assert from "node:assert/strict";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import {
  pluginFindings,
  snapshotCases,
  snapshotDocument,
  snapshotSuite,
} from "./diagnostic-snapshot.mjs";
import { messages, require } from "./helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const typescript = { parser: tsParser, sourceType: "module", globals: {} };

const {
  createAliasScopeTracker,
} = require("../src/rules/no-banned-import-outside-allowed-paths-scopes.js");
const {
  recordAssignmentTag,
  recordVariableTag,
} = require("../src/rules/no-banned-import-outside-allowed-paths-aliases.js");
const { matchDirectMockCallApply } = require("../src/rules/module-mock-call-apply.js");

describe("lint early-return edges", () => {
  it("snapshots remaining diagnostics", async () => {
    await expect(
      snapshotDocument({
        "nextjs-metadata-exports-location": snapshotSuite(
          "nextjs-metadata-exports-location",
          languageOptions,
          {
            valid: [
              {
                code: "export const metadata = {};",
                filename: "lib/meta.ts",
                languageOptions: typescript,
              },
            ],
          },
        ),
        "nextjs-no-manual-script-tags": snapshotSuite(
          "nextjs-no-manual-script-tags",
          languageOptions,
          {
            valid: [
              { code: "<div />;", filename: "components/x.jsx" },
              {
                code: '<script type="application/ld+json"></script>;',
                filename: "app/page.tsx",
                languageOptions: typescript,
              },
              {
                code: '<script id="allowed" dangerouslySetInnerHTML={{ __html: "" }}></script>;',
                filename: "app/page.tsx",
                languageOptions: typescript,
                options: [
                  { allowInlineScriptIds: ["allowed"], allowInlineScriptIdPatterns: ["("] },
                ],
              },
            ],
          },
        ),
        "no-delete-property": snapshotSuite("no-delete-property", languageOptions, {
          valid: [{ code: "const ok = !value;", filename: "fixture.jsx" }],
          invalid: [
            {
              code: "delete (value as Thing).prop;",
              filename: "a.tsx",
              languageOptions: typescript,
            },
          ],
        }),
        "no-placeholder-never-type-exports": snapshotSuite(
          "no-placeholder-never-type-exports",
          languageOptions,
          {
            valid: [
              {
                code: "type Keep = string; export type { Keep };",
                filename: "a.ts",
                languageOptions: typescript,
              },
            ],
            invalid: [
              {
                code: 'type Placeholder = never; export type { Placeholder as "alias" };',
                filename: "a.ts",
                languageOptions: typescript,
              },
            ],
          },
        ),
      }),
    ).toMatchFileSnapshot("snapshots/coverage-lint-edges-remaining.snap");
  }, 30_000);

  it("covers next metadata and script guard paths", () => {
    messages(
      'export { metadata as "meta" };',
      "nextjs-metadata-exports-location",
      undefined,
      "app/page.tsx",
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

  it("snapshots playwright diagnostics", async () => {
    const cases = [
      {
        file: "fixture.jsx",
        findings: pluginFindings(
          "export function button() { return <button />; }",
          "playwright-require-exported-component-attribute",
        ),
        name: "lowercase button",
        rule: "playwright-require-exported-component-attribute",
      },
      {
        file: "fixture.jsx",
        findings: pluginFindings(
          'export function Button() { return <button data-pw="ok" />; }',
          "playwright-require-exported-component-attribute",
        ),
        name: "Button data-pw",
        rule: "playwright-require-exported-component-attribute",
      },
    ];
    await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
      "snapshots/coverage-lint-edges.snap",
    );
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
    const result = expectMatch(
      matchDirectMockCallApply(
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
      ),
    );
    assert.equal(result.specifierNode, "mod");
    assert.equal(result.factory.type, "ArrowFunctionExpression");
    const applyIdentifier = expectMatch(
      matchDirectMockCallApply(
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
      ),
    );
    assert.equal(applyIdentifier.specifierNode, undefined);
    assert.equal(applyIdentifier.factory, undefined);
    const callMatch = expectMatch(
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
    assert.equal(callMatch.specifierNode, "mod");
    assert.equal(callMatch.factory.type, "ArrowFunctionExpression");
  });
});

function expectDefined(value) {
  assert.ok(value !== undefined);
}

function expectMatch(value) {
  assert.ok(value, "expected a match result");
  assert.ok(value.mock, "expected a resolved mock");
  return value;
}
