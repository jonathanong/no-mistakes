import assert from "node:assert/strict";
import { describe, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { messages, require } from "./helpers.mjs";

const { createReactNodeFacts } = require("../src/react-node-types.js");
const {
  definePattern,
  keyName,
  qualifierText,
  reactTypeStatus,
  typeName,
} = require("../src/react-node-type-query.js");
const { logicalCall } = require("../src/rules/logical-call.js");
const {
  importFact,
  importSpellings,
  reportsUnanalyzable,
  rootIdentifier,
  typeOnlyImport,
} = require("../src/rules/import-resolution.js");
const { nextImportState } = require("../src/rules/nextjs-script-imports.js");
const nullableFetch = require("../src/rules/server-require-nullable-fetch-wrapper.js");
const { classifiedProps } = require("../src/rules/react-nullish-props.js");
const { patternKey, reactUseStatus, staticName } = require("../src/rules/react-use-callee.js");

const facts = {
  aliases: new Map(),
  reactNodeNames: new Set(["React.ReactNode"]),
  unresolvedNamespaces: new Set(["Missing"]),
  unresolvedTypes: new Set(["Lost"]),
};

function probe(code, filename = "probe.tsx") {
  const facts = [];
  const linter = new Linter({ configType: "flat", cwd: "/tmp" });
  linter.verify(
    code,
    {
      files: ["**"],
      languageOptions: {
        ecmaVersion: 2024,
        sourceType: "module",
        parser: tsParser,
        parserOptions: { ecmaFeatures: { jsx: true } },
      },
      plugins: {
        local: {
          rules: {
            probe: {
              create(context) {
                return {
                  "Identifier, CallExpression, MemberExpression, VariableDeclarator"(node) {
                    const fact = importFact(node, context);
                    facts.push(fact);
                    importSpellings(
                      rootIdentifier(node),
                      node.type === "Identifier" ? node.name : "root.member",
                      context,
                    );
                  },
                };
              },
            },
          },
        },
      },
      rules: { "local/probe": "error" },
    },
    filename,
  );
  return facts;
}

function factsFor(code) {
  const byName = new Map();
  const calls = [];
  const linter = new Linter({ configType: "flat", cwd: "/tmp" });
  linter.verify(
    code,
    {
      files: ["**"],
      languageOptions: {
        ecmaVersion: 2024,
        sourceType: "module",
        parser: tsParser,
        parserOptions: { ecmaFeatures: { jsx: true } },
      },
      plugins: {
        local: {
          rules: {
            probe: {
              create(context) {
                return {
                  Identifier(node) {
                    const list = byName.get(node.name) ?? [];
                    list.push(importFact(node, context));
                    byName.set(node.name, list);
                  },
                  CallExpression(node) {
                    calls.push(importFact(node, context));
                  },
                };
              },
            },
          },
        },
      },
      rules: { "local/probe": "error" },
    },
    "bindings.ts",
  );
  return { byName, calls };
}

describe("typescript import coverage", () => {
  it("covers resolver edges that are not imports", () => {
    assert.equal(reportsUnanalyzable({}), true);
    assert.equal(reportsUnanalyzable({ options: [{ unanalyzable: "ignore" }] }), false);
    assert.equal(typeOnlyImport(null), false);
    assert.equal(typeOnlyImport({ node: { importKind: "type" } }), true);
    assert.equal(typeOnlyImport({ parent: { importKind: "type" } }), true);
    assert.equal(rootIdentifier(null), null);
    assert.equal(rootIdentifier({ type: "Literal" }), null);
    assert.equal(
      rootIdentifier({
        type: "MemberExpression",
        object: { type: "Identifier", name: "root" },
        property: { type: "Identifier", name: "member" },
      }).name,
      "root",
    );
    assert.deepEqual(importFact(null, {}), null);
    assert.deepEqual(importFact({ type: "Literal" }, { sourceCode: {} }), null);
    assert.deepEqual(importSpellings(null, "root", {}), { names: [], unresolved: false });

    const seen = probe(`
      import type { use as typed } from "react";
      import { read as load } from "./w";
      import * as React from "react";
      import Def from "react";
      import Eq = require("react");
      import Local = React;
      import Other = require("left-pad");
      const { use: taken = unused } = require("react");
      const { default: fallback } = React;
      const { ["use"]: computed } = React;
      const { [dynamic]: missing } = require(moduleName);
      const direct = require("react");
      const alias = direct;
      const renamed = load;
      function require() { return "nope"; }
      const shadowed = require("react");
      var plain = 1;
      var plain = 2;
      var open;
      var open = plain;
      function over(id: string): void;
      function over(id: string, extra: boolean): void;
      function over(id: string) {}
      import { clash } from "./w";
      function clash() {}
      typed;
      load.member;
      taken;
      computed;
      missing;
      alias;
      renamed.member;
      shadowed;
      over(1);
      clash();
    `);
    assert.ok(seen.some((fact) => fact && fact.module === "react" && fact.kind === "namespace"));
    assert.ok(seen.some((fact) => fact && fact.unresolved));

    const resolved = factsFor(`
      require();
      require("react", extra);
      const { other: skipped, use: taken } = require("react");
      const { use: named, ...rest } = require("react");
      declare const mod: string;
      const base = require(mod);
      const { use: fromBase } = base;
      import { read } from "./local";
      const { child: fromNamed } = read;
      var mixed = require("react");
      var mixed = 1;
      const cycle = cycle;
      taken;
      rest;
      fromBase;
      fromNamed;
      mixed;
      cycle;
    `);
    const has = (name, predicate) => resolved.byName.get(name).some(predicate);
    assert.equal(
      resolved.calls.some((fact) => fact && fact.unresolved),
      true,
    );
    assert.equal(
      has("taken", (fact) => fact && fact.module === "react" && fact.imported === "use"),
      true,
    );
    assert.equal(
      has("fromBase", (fact) => fact && fact.unresolved),
      true,
    );
    assert.equal(
      has("rest", (fact) => fact === null),
      true,
    );
    assert.equal(
      has(
        "fromNamed",
        (fact) =>
          fact &&
          fact.module === "./local" &&
          fact.name === "read.child" &&
          fact.imported === "child",
      ),
      true,
    );
    assert.equal(
      has("mixed", (fact) => fact && fact.unresolved),
      true,
    );
    assert.equal(
      has("cycle", (fact) => fact && fact.unresolved),
      true,
    );
  });

  it("covers ReactNode, script, and use edges", () => {
    assert.equal(keyName(null), null);
    assert.equal(keyName({ type: "Identifier", name: "child" }), "child");
    assert.equal(keyName({ type: "Literal", value: "child" }), "child");
    assert.equal(keyName({ type: "TemplateLiteral" }), null);
    assert.equal(qualifierText(null), null);
    assert.equal(qualifierText({ type: "Identifier", name: "ReactNode" }), "ReactNode");
    assert.equal(
      qualifierText({
        type: "TSQualifiedName",
        left: { type: "Identifier", name: "React" },
        right: { type: "Identifier", name: "ReactNode" },
      }),
      "React.ReactNode",
    );
    assert.equal(
      qualifierText({
        type: "TSQualifiedName",
        left: { type: "Literal" },
        right: { type: "Identifier", name: "ReactNode" },
      }),
      null,
    );
    assert.equal(typeName(null), null);
    assert.equal(typeName({ type: "TSLiteralType" }), null);
    assert.equal(
      typeName({
        type: "TSImportType",
        source: { type: "Literal", value: "react" },
        qualifier: { type: "Identifier", name: "ReactNode" },
      }),
      "React.ReactNode",
    );
    assert.equal(typeName({ type: "TSTypeReference", typeName: { type: "Literal" } }), null);
    assert.equal(reactTypeStatus(null, null), "no");
    assert.equal(
      reactTypeStatus(
        {
          type: "TSUnionType",
          types: [{ type: "TSTypeReference", typeName: { type: "Identifier", name: "Lost" } }],
        },
        facts,
      ),
      "unresolved",
    );
    assert.equal(
      reactTypeStatus(
        {
          type: "TSTypeReference",
          typeName: {
            type: "TSQualifiedName",
            left: { type: "Identifier", name: "Missing" },
            right: { type: "Identifier", name: "Other" },
          },
        },
        facts,
      ),
      "no",
    );
    assert.equal(
      reactTypeStatus(
        {
          type: "TSTypeReference",
          typeName: {
            type: "TSQualifiedName",
            left: { type: "Identifier", name: "Missing" },
            right: { type: "Identifier", name: "ReactNode" },
          },
        },
        facts,
      ),
      "unresolved",
    );
    assert.equal(
      reactTypeStatus(
        {
          type: "TSUnionType",
          types: [
            { type: "TSTypeReference", typeName: { type: "Identifier", name: "Lost" } },
            {
              type: "TSTypeReference",
              typeName: { type: "Identifier", name: "React.ReactNode" },
            },
          ],
        },
        facts,
      ),
      "yes",
    );
    assert.equal(
      reactTypeStatus(
        {
          type: "TSImportType",
          source: { type: "Literal", value: "left-pad", raw: '"left-pad"' },
          qualifier: { type: "Identifier", name: "ReactNode" },
        },
        facts,
      ),
      "no",
    );
    assert.equal(
      typeName({
        type: "TSImportType",
        argument: {
          type: "TSLiteralType",
          literal: { type: "Literal", value: "react", raw: '"react"' },
        },
        qualifier: { type: "Identifier", name: "ReactNode" },
      }),
      "React.ReactNode",
    );
    const defined = [];
    definePattern(
      {
        type: "ObjectPattern",
        properties: [
          { type: "RestElement" },
          {
            type: "Property",
            computed: true,
            key: { type: "Literal", value: "child", raw: '"child"' },
            value: { type: "AssignmentPattern", left: { type: "Identifier", name: "child" } },
          },
          {
            type: "Property",
            computed: false,
            key: { type: "Identifier", name: "lost" },
            value: { type: "Identifier", name: "lost" },
          },
        ],
      },
      new Set(["child"]),
      (name) => defined.push(name),
      () => {},
      new Set(["lost"]),
      (name) => defined.push(`unresolved:${name}`),
    );
    definePattern(null, null, () => {});
    assert.deepEqual(defined, ["child", "lost", "unresolved:lost"]);

    const nodeFacts = createReactNodeFacts({
      body: [
        {
          type: "TSImportEqualsDeclaration",
          id: { type: "Identifier", name: "Local" },
          moduleReference: { type: "TSQualifiedName" },
        },
        {
          type: "TSTypeAliasDeclaration",
          id: { type: "Identifier", name: "Alias" },
          typeAnnotation: {
            type: "TSTypeReference",
            typeName: {
              type: "TSQualifiedName",
              left: { type: "Identifier", name: "Local" },
              right: { type: "Identifier", name: "ReactNode" },
            },
          },
        },
        {
          type: "TSInterfaceDeclaration",
          id: { type: "Identifier", name: "Box" },
          body: {
            body: [
              {
                type: "TSPropertySignature",
                key: { type: "TemplateLiteral", expressions: [], quasis: [] },
                typeAnnotation: {
                  type: "TSTypeAnnotation",
                  typeAnnotation: {
                    type: "TSTypeReference",
                    typeName: { type: "Identifier", name: "React.ReactNode" },
                  },
                },
              },
            ],
          },
          extends: [],
        },
        {
          type: "TSImportEqualsDeclaration",
          id: { type: "Identifier", name: "Other" },
          moduleReference: {
            type: "TSExternalModuleReference",
            expression: { type: "Literal", value: "left-pad", raw: '"left-pad"' },
          },
        },
        {
          type: "ExportNamedDeclaration",
          declaration: {
            type: "VariableDeclaration",
            kind: "const",
            declarations: [
              {
                type: "VariableDeclarator",
                id: { type: "Identifier", name: "ReactNs" },
                init: {
                  type: "CallExpression",
                  callee: { type: "Identifier", name: "require" },
                  arguments: [{ type: "Literal", value: "react", raw: '"react"' }],
                },
              },
            ],
          },
        },
      ],
    });
    assert.equal(nodeFacts.reactNodeNames.has("ReactNs.ReactNode"), true);
    assert.equal(nodeFacts.unresolvedTypes.has("Alias"), true);
    assert.equal(nodeFacts.objectProps.get("Box")?.size ?? 0, 0);
    assert.deepEqual(
      classifiedProps(
        {
          type: "TSTypeLiteral",
          members: [
            {
              type: "TSPropertySignature",
              key: { type: "TemplateLiteral", expressions: [], quasis: [] },
              typeAnnotation: {
                type: "TSTypeAnnotation",
                typeAnnotation: {
                  type: "TSTypeReference",
                  typeName: { type: "Identifier", name: "React.ReactNode" },
                },
              },
            },
          ],
        },
        facts,
      ).props.size,
      0,
    );

    const loop = { type: "Program", body: [] };
    loop.body.push(loop);
    const state = nextImportState(loop, { sourceCode: { getScope() {} } });
    const imported = nextImportState(
      {
        type: "Program",
        body: [
          { type: "ImportExpression" },
          {
            type: "TSImportEqualsDeclaration",
            moduleReference: { type: "TSQualifiedName" },
          },
          {
            type: "TSImportEqualsDeclaration",
            id: { type: "Identifier", name: "Head" },
            moduleReference: { type: "TSQualifiedName" },
          },
        ],
      },
      { sourceCode: { getScope() {} } },
    );
    assert.equal(state.next, false);
    assert.equal(imported.unresolved, true);

    assert.equal(staticName(null, false), null);
    assert.equal(
      patternKey(
        {
          type: "ObjectPattern",
          properties: [
            { type: "RestElement" },
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "label" },
              value: { type: "Literal", value: 1 },
            },
            {
              type: "Property",
              computed: true,
              key: { type: "Literal", value: "use", raw: '"use"' },
              value: { type: "Identifier", name: "taken" },
            },
          ],
        },
        "taken",
      ),
      "use",
    );
    assert.equal(reactUseStatus(null, { sourceCode: { getScope() {} } }, new Set()), "no");
    assert.deepEqual(
      logicalCall(
        {
          type: "CallExpression",
          optional: false,
          callee: {
            type: "MemberExpression",
            computed: false,
            optional: false,
            object: { type: "Identifier", name: "fn" },
            property: { type: "Identifier", name: "apply" },
          },
          arguments: [
            { type: "Identifier", name: "receiver" },
            { type: "Literal", value: null, raw: "null" },
          ],
        },
        undefined,
        true,
      ).arguments,
      [],
    );
    assert.equal(nullableFetch.__test.functionName({ type: "FunctionDeclaration" }), null);

    assert.deepEqual(
      messages(
        'import { type use } from "react";\nuse(Promise.resolve(1));',
        "react-no-use-promise-resolve",
        undefined,
        "use.tsx",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        'import * as React from "react";\nconst { label: taken } = React;\ntaken(Promise.resolve(1));',
        "react-no-use-promise-resolve",
        undefined,
        "use.tsx",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        "declare const mod: string;\nfunction A(props: { child: import(mod).ReactNode; [k: string]: string; run(): void }) {\n  const node = props.child;\n  const { child } = props;\n  return child ?? node ?? props.child ?? foo().child ?? null;\n}",
        "react-no-nullish-react-node",
        undefined,
        "node.tsx",
      ).includes("unanalyzable"),
      true,
    );
    assert.deepEqual(
      messages(
        'declare const nextModule: string;\nimport(nextModule);\nexport const A = () => <script src="/a.js" />;',
        "nextjs-no-manual-script-tags",
        undefined,
        "src/widget.tsx",
      ),
      ["unanalyzable"],
    );
    assert.deepEqual(
      messages("<script src='/a.js' />", "nextjs-no-manual-script-tags", undefined, ""),
      [],
    );
    assert.deepEqual(
      messages(
        "declare const mod: string;\nconst use = require(mod);\nuse(Promise.resolve(1));",
        "react-no-use-promise-resolve",
        undefined,
        "dyn.tsx",
      ),
      ["unanalyzable"],
    );
    assert.deepEqual(
      messages(
        'import * as React from "react";\nconst { use: taken } = React;\ntaken(Promise.resolve(1));',
        "react-no-use-promise-resolve",
        undefined,
        "ns.tsx",
      ),
      ["resolve"],
    );
    assert.deepEqual(
      messages(
        "var called = 1;\nvar called = 2;\ncalled(Promise.resolve(1));",
        "react-no-use-promise-resolve",
        undefined,
        "redeclare.tsx",
      ),
      [],
    );
    assert.deepEqual(
      messages("foo().child ?? null;", "react-no-nullish-react-node", undefined, "call.tsx"),
      [],
    );
    const nullableIds = messages(
      `declare const mod: string;
declare const api: { getWidget(): Promise<Widget | null> };
const mystery = require(mod);
export async function read(): Promise<Widget | null> {
  console.log(api.getWidget());
  mystery.call(...[api.getWidget()]);
  return mystery(api.getWidget());
}`,
      "server-require-nullable-fetch-wrapper",
      {
        getterCalleePatterns: ["^api\\.getWidget$"],
        requiredWrapperCallee: "wrapNullable",
        nullableReturnTypeNames: ["Widget"],
      },
      "read.ts",
    );
    assert.equal(nullableIds.includes("wrapper"), true);
    assert.equal(nullableIds.includes("unanalyzable"), true);
  });
});
