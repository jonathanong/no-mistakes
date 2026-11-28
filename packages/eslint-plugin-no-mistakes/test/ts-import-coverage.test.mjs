import assert from "node:assert/strict";
import { describe, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { messages, require } from "./helpers.mjs";

const { createReactNodeFacts } = require("../src/react-node-types.js");
const {
  assertedType,
  definePattern,
  keyName,
  qualifierText,
  reactTypeStatus,
  typeName,
} = require("../src/react-node-type-query.js");
const { logicalCall } = require("../src/rules/logical-call.js");
const {
  factAtPath,
  namespaceMember,
  patternDefault,
  patternPath,
} = require("../src/rules/import-pattern-path.js");
const {
  importFact,
  importSpellings,
  reportsUnanalyzable,
  requireModule,
  rootIdentifier,
  typeOnlyImport,
} = require("../src/rules/import-resolution.js");
const { nextImportState } = require("../src/rules/nextjs-script-imports.js");
const nullableFetch = require("../src/rules/server-require-nullable-fetch-wrapper.js");
const { reassignedBinding } = require("../src/rules/react-nullish-binding-status.js");
const { localTypeStatus } = require("../src/react-node-local-type.js");
const { nearestDeclarations } = require("../src/rules/react-nullish-declarations.js");
const { laterBindingStatus } = require("../src/rules/react-nullish-later-status.js");
const { classifiedProps } = require("../src/rules/react-nullish-props.js");
const {
  UNSUBSTITUTED,
  appliedStatus,
  intersectedProps,
} = require("../src/rules/react-nullish-substitutions.js");
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

  it("covers helper branches across ReactNode, substitution, and import path resolution", () => {
    assert.equal(
      appliedStatus(
        { type: "TSTypeReference", typeName: { type: "Identifier", name: "T" } },
        facts,
        new Map([[{ name: { name: "T" } }, UNSUBSTITUTED]]),
      ),
      "unresolved",
    );
    assert.deepEqual(
      intersectedProps(null, () => null),
      { props: null, unresolved: null },
    );
    assert.deepEqual(
      intersectedProps([1, 2, 3], (item) => {
        if (item === 1) return { props: null, unresolved: null };
        if (item === 2) return { props: new Set(["a"]), unresolved: new Set(["b"]) };
        return { props: new Set(), unresolved: new Set(["a", "c"]), declared: new Set(["a", "c"]) };
      }),
      {
        props: new Set(),
        unresolved: new Set(["a", "b", "c"]),
        declared: new Set(["a", "b", "c"]),
      },
    );
    assert.deepEqual(
      intersectedProps([1, 2], (item) =>
        item === 1
          ? { props: null, unresolved: new Set(["u"]) }
          : { props: new Set(["p"]), unresolved: null },
      ),
      {
        props: new Set(["p"]),
        unresolved: new Set(["u"]),
        declared: new Set(["u", "p"]),
      },
    );

    assert.equal(namespaceMember(null, "use"), null);
    assert.deepEqual(namespaceMember({ unresolved: true }, "use"), { unresolved: true });
    assert.deepEqual(factAtPath({ unresolved: true }, ["use"], namespaceMember), {
      unresolved: true,
    });
    assert.equal(
      factAtPath({ kind: "other" }, ["use"], () => null),
      null,
    );
    assert.deepEqual(
      patternPath(
        {
          type: "ObjectPattern",
          properties: [
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "nested" },
              value: {
                type: "ObjectPattern",
                properties: [
                  {
                    type: "Property",
                    computed: false,
                    key: { type: "Identifier", name: "other" },
                    value: { type: "Identifier", name: "other" },
                  },
                ],
              },
            },
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "target" },
              value: { type: "Identifier", name: "target" },
            },
          ],
        },
        "target",
        () => null,
      ),
      ["target"],
    );
    assert.equal(patternDefault(null, "target"), null);
    assert.deepEqual(
      patternDefault(
        {
          type: "AssignmentPattern",
          left: {
            type: "ObjectPattern",
            properties: [
              {
                type: "Property",
                computed: true,
                key: { type: "Identifier", name: "dyn" },
                value: { type: "Identifier", name: "target" },
              },
            ],
          },
          right: { type: "Identifier", name: "fallback" },
        },
        "target",
      ),
      { type: "ObjectExpression", properties: [] },
    );

    assert.equal(
      requireModule(
        {
          type: "CallExpression",
          callee: { type: "Identifier", name: "require" },
          arguments: [{ type: "Literal", value: "react", raw: '"react"' }],
        },
        {},
      ),
      "react",
    );
    assert.equal(
      assertedType({
        type: "SequenceExpression",
        expressions: [
          { type: "Literal", value: 0 },
          {
            type: "TSAsExpression",
            expression: { type: "Identifier", name: "x" },
            typeAnnotation: {
              type: "TSTypeReference",
              typeName: { type: "Identifier", name: "T" },
            },
          },
        ],
      })?.typeName?.name,
      "T",
    );
    assert.equal(
      reassignedBinding(
        { type: "Identifier", name: "missing" },
        {
          sourceCode: {
            getScope() {
              return null;
            },
          },
        },
      ),
      false,
    );
    assert.equal(
      patternKey(
        {
          type: "ObjectPattern",
          properties: [
            {
              type: "Property",
              computed: true,
              key: { type: "Identifier", name: "dyn" },
              value: { type: "Identifier", name: "u" },
            },
          ],
        },
        "u",
      ),
      undefined,
    );

    const syntheticFacts = createReactNodeFacts({
      body: [
        { type: "ImportDeclaration", source: { value: "react" } },
        {
          type: "TSImportEqualsDeclaration",
          id: { type: "Identifier", name: "React" },
          moduleReference: {
            type: "TSExternalModuleReference",
            expression: { type: "Literal", value: "react", raw: '"react"' },
          },
        },
        {
          type: "VariableDeclaration",
          kind: "const",
          declarations: [
            {
              type: "VariableDeclarator",
              id: { type: "Identifier", name: "badArity" },
              init: {
                type: "CallExpression",
                callee: { type: "Identifier", name: "require" },
                arguments: [],
              },
            },
            {
              type: "VariableDeclarator",
              id: { type: "Identifier", name: "notReq" },
              init: {
                type: "CallExpression",
                callee: { type: "Identifier", name: "loader" },
                arguments: [{ type: "Literal", value: "react", raw: '"react"' }],
              },
            },
            {
              type: "VariableDeclarator",
              id: { type: "Identifier", name: "dynReq" },
              init: {
                type: "CallExpression",
                callee: { type: "Identifier", name: "require" },
                arguments: [{ type: "Identifier", name: "mod" }],
              },
            },
            {
              type: "VariableDeclarator",
              id: { type: "Identifier", name: "React" },
              init: {
                type: "CallExpression",
                callee: { type: "Identifier", name: "require" },
                arguments: [{ type: "Literal", value: "react", raw: '"react"' }],
              },
            },
          ],
        },
      ],
    });
    assert.equal(syntheticFacts.unresolvedNamespaces.has("dynReq"), true);
    const foreignReactFacts = createReactNodeFacts({
      body: [
        {
          type: "VariableDeclaration",
          kind: "const",
          declarations: [
            {
              type: "VariableDeclarator",
              id: { type: "Identifier", name: "React" },
              init: {
                type: "CallExpression",
                callee: { type: "Identifier", name: "require" },
                arguments: [{ type: "Literal", value: "left-pad", raw: '"left-pad"' }],
              },
            },
          ],
        },
      ],
    });
    assert.equal(foreignReactFacts.reactNodeNames.has("React.ReactNode"), false);

    assert.deepEqual(
      messages(
        `import type { ReactNode } from "react";
declare const mod: string;
type Unresolved = import(mod).ReactNode;
type MissingDefault<T, U = string> = T;
export function missingDefault(v: MissingDefault) { return v ?? null; }
export function shadowKnown<ReactNode>(v: ReactNode) { return v ?? null; }
export function forwardBranches(src: { child: ReactNode; lost: Unresolved; plain: string }, plainObj: { label: string }, dynKey: string) {
  const r1 = () => a ?? null;
  const r2 = () => b ?? null;
  const r3 = () => c ?? null;
  const r4 = () => d ?? null;
  const r5 = () => e ?? null;
  const r6 = () => f ?? null;
  const r7 = () => g ?? null;
  const r8 = () => h ?? null;
  const r9 = () => i ?? null;
  const r10 = () => j ?? null;
  const { ...rest, other, child: a = null } = src as { child: ReactNode };
  const { [dynKey]: b } = src;
  const { [dynKey]: c } = plainObj;
  const { lost: d } = src;
  const { plain: e } = src;
  const { [dynKey]: f }: { child: ReactNode } = src;
  const { [dynKey]: g }: { label: string } = plainObj;
  const { lost: h }: { lost: Unresolved } = src;
  let { child: i } = src;
  i = "ok";
  const j: Unresolved = src.lost;
  return [r1(), r2(), r3(), r4(), r5(), r6(), r7(), r8(), r9(), r10(), rest, other];
}
export function forwardRecorded(src: { child: ReactNode; lost: Unresolved; plain: string }, empty: { label: string }, dynKey: string) {
  const alias = src;
  const emptyAlias = empty;
  const r1 = () => a ?? null;
  const r2 = () => b ?? null;
  const r3 = () => c ?? null;
  const r4 = () => d ?? null;
  const r5 = () => e ?? null;
  const { child: a } = alias;
  const { lost: b } = alias;
  const { plain: c } = alias;
  const { [dynKey]: d } = alias;
  const { [dynKey]: e } = emptyAlias;
  return [r1(), r2(), r3(), r4(), r5()];
}
export function forwardCycle() {
  const r = () => a ?? null;
  const a = b;
  const b = a;
  return r();
}
interface TopBase { child: ReactNode; lost: Unresolved; }
export function localHeritage(x: any) {
  interface Self extends Self { a: string }
  interface NsExt extends ns.Base { a: string }
  interface Sub extends TopBase {}
  const f1 = (p: Self) => p.a ?? "";
  const f2 = (p: NsExt) => p.a ?? "";
  const f3 = (p: Sub) => (p.child ?? null) || (p.lost ?? null);
  const f4 = <T,>(p: T) => (p as T) ?? null;
  return [f1(x), f2(x), f3(x), f4(x)];
}`,
        "react-no-nullish-react-node",
        undefined,
        "branch-coverage.tsx",
      ).includes("nullish"),
      true,
    );

    assert.deepEqual(
      messages(
        `import * as React from "react";
import { "use" as stringUse } from "react";
declare const dyn: string;
declare const other: Record<string, any>;
let reassigned = stringUse;
reassigned = () => 1;
const { [dyn]: dynFromReact } = React;
const { [dyn]: dynFromOther } = other;
const cyc = cyc;
let letAlias = React.use;
letAlias = () => 1;
export function run(args: [Promise<number>]) {
  reassigned(Promise.resolve(1));
  dynFromReact(Promise.resolve(1));
  dynFromOther(Promise.resolve(1));
  cyc(Promise.resolve(1));
  letAlias(Promise.resolve(1));
  stringUse(...args);
}`,
        "react-no-use-promise-resolve",
        undefined,
        "use-branch-coverage.tsx",
      ).includes("unanalyzable"),
      true,
    );

    const nullScope = {
      sourceCode: {
        getScope() {
          return null;
        },
      },
    };
    assert.equal(
      reactUseStatus({ type: "Identifier", name: "use" }, nullScope, new Set(["use"])),
      "yes",
    );

    const switchNode = { type: "SwitchStatement" };
    const caseNode = { type: "SwitchCase", parent: switchNode };
    const refInSwitch = { type: "TSTypeReference", parent: switchNode };
    const refInCase = { type: "TSTypeReference", parent: caseNode };
    assert.equal(localTypeStatus(refInSwitch, "Slot", facts, new Set(), reactTypeStatus), null);
    assert.equal(localTypeStatus(refInCase, "Slot", facts, new Set(), reactTypeStatus), null);
    assert.equal(nearestDeclarations(refInSwitch, "Slot"), null);
    assert.equal(nearestDeclarations(refInCase, "Slot"), null);
    const switchWithBareCase = { type: "SwitchStatement", cases: [{ type: "SwitchCase" }] };
    const refInBareSwitch = { type: "TSTypeReference", parent: switchWithBareCase };
    assert.equal(localTypeStatus(refInBareSwitch, "Slot", facts, new Set(), reactTypeStatus), null);
    assert.equal(nearestDeclarations(refInBareSwitch, "Slot"), null);

    const mockScope = (variable) => ({
      sourceCode: {
        getScope() {
          return {
            variables: [variable],
            set: new Map([[variable.name, variable]]),
            upper: null,
          };
        },
      },
    });
    const fakeId = { type: "Identifier", name: "x", range: [0, 1] };
    const fakeInitId = { type: "Identifier", name: "src", range: [20, 23] };
    const paramDef = { type: "Parameter", node: { range: [10, 15] } };
    const objPatDef = {
      type: "Variable",
      node: {
        range: [10, 30],
        id: {
          type: "ObjectPattern",
          properties: [
            {
              type: "Property",
              computed: true,
              key: { type: "Identifier", name: "dyn" },
              value: { type: "Identifier", name: "x" },
            },
          ],
        },
        init: fakeInitId,
      },
    };
    const multiVar = {
      name: "x",
      defs: [paramDef, objPatDef],
      references: [],
    };
    assert.equal(
      laterBindingStatus(
        fakeId,
        mockScope(multiVar),
        reactTypeStatus,
        () => "no",
        facts,
        new Set(),
        (_node, kind) => (kind === "unresolvedMembers" ? new Set(["lost"]) : null),
      ),
      "unresolved",
    );
    assert.equal(
      laterBindingStatus(
        fakeId,
        mockScope(multiVar),
        reactTypeStatus,
        () => "no",
        facts,
        new Set(),
        (_node, kind) => (kind === "unresolvedMembers" ? new Set() : null),
      ),
      "no",
    );
    const nonMatchingPatVar = {
      name: "x",
      defs: [
        {
          type: "Variable",
          node: {
            range: [10, 30],
            id: {
              type: "ObjectPattern",
              properties: [
                {
                  type: "Property",
                  computed: false,
                  key: { type: "Identifier", name: "other" },
                  value: { type: "Identifier", name: "other" },
                },
              ],
            },
            init: {
              type: "Identifier",
              name: "typedInit",
              typeAnnotation: {
                type: "TSTypeAnnotation",
                typeAnnotation: { type: "TSTypeLiteral", members: [] },
              },
            },
          },
        },
      ],
      references: [],
    };
    assert.equal(
      laterBindingStatus(
        fakeId,
        mockScope(nonMatchingPatVar),
        reactTypeStatus,
        () => "no",
        facts,
        new Set(),
        null,
      ),
      "no",
    );
    const directTypedInitVar = {
      name: "x",
      defs: [
        {
          type: "Variable",
          node: {
            range: [10, 30],
            id: {
              type: "ObjectPattern",
              properties: [
                {
                  type: "Property",
                  computed: false,
                  key: { type: "Identifier", name: "child" },
                  value: { type: "Identifier", name: "x" },
                },
              ],
            },
            init: {
              type: "Identifier",
              name: "typedInit",
              typeAnnotation: {
                type: "TSTypeAnnotation",
                typeAnnotation: { type: "TSTypeLiteral", members: [] },
              },
            },
          },
        },
      ],
      references: [],
    };
    assert.equal(
      laterBindingStatus(
        fakeId,
        mockScope(directTypedInitVar),
        reactTypeStatus,
        () => "no",
        facts,
        new Set(),
        null,
      ),
      "no",
    );

    assert.deepEqual(
      messages(
        `import type { ReactNode } from "react";
type Slot = ReactNode;
type Loop<T extends T> = T;
export function loopParam(v: Loop) { return v ?? null; }
export function switchCaseTest(flag: any) {
  switch (flag) {
    case (flag as Slot):
      type Slot = string;
      return "a";
  }
  return "b";
}
export function forwardExtras(src: { child: ReactNode }) {
  var multi = src;
  var multi = src;
  const r1 = () => k ?? null;
  const r2 = () => m ?? null;
  const r3 = () => arr ?? null;
  const { ["child"]: k } = src;
  const { child: m } = multi;
  const [arr] = [src.child];
  return [r1(), r2(), r3()];
}
export function cyclicIfaces(x: any) {
  interface C extends D {}
  interface D extends C {}
  return (x as C).a ?? "";
}`,
        "react-no-nullish-react-node",
        undefined,
        "extra-branches.tsx",
      ).includes("nullish"),
      true,
    );

    assert.deepEqual(
      messages(
        `import * as React from "react";
import type { use as typeUse } from "react";
const { use: fromType } = typeUse;
const a = b.call;
const b = a.call;
let letCall = React.use.call;
let mut: any;
mut++;
let destr: any;
({ a: destr } = require("react"));
export function run() {
  fromType(Promise.resolve(1));
  a(Promise.resolve(1));
  letCall(Promise.resolve(1));
  mut(Promise.resolve(1));
  destr(Promise.resolve(1));
}`,
        "react-no-use-promise-resolve",
        undefined,
        "use-extra-branches.tsx",
      ).includes("unanalyzable"),
      true,
    );
  });
});
