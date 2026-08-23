import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { require } from "./helpers.mjs";

const {
  DEFAULT_CURSOR_INCLUDE,
  matchesCursorFile,
  resolveCursorContractOptions,
} = require("../src/rules/postgres-cursor-options");
const {
  findVariable,
  normalizeFilename,
  propertyName,
  unwrap,
} = require("../src/rules/postgres-cursor-ast");
const {
  isCursorExecutor,
  isCursorModule,
  namedCursorImport,
  namespaceCursorImport,
  namespaceCursorMember,
  namespaceImportMember,
} = require("../src/rules/postgres-cursor-imports");

const helpers = { findVariable, propertyName, unwrap };
const config = { modules: new Set(["@db/cursors"]), executors: new Set(["runCursor"]) };
const options = { modules: ["@db/cursors"], executors: ["runCursor"] };

function identifier(name) {
  return { type: "Identifier", name };
}

function importVariable(name, specifierType, extras = {}) {
  const specifier = {
    type: specifierType,
    importKind: extras.importKind ?? "value",
    imported: { name: extras.imported ?? name, type: "Identifier" },
  };
  const declaration = {
    type: "ImportDeclaration",
    importKind: extras.declarationKind ?? "value",
    source: { type: "Literal", value: extras.module ?? "@db/cursors" },
  };
  specifier.parent = declaration;
  return {
    name,
    defs: [{ type: "ImportBinding", node: specifier, parent: declaration }],
    references: [],
  };
}

function contextFor(variable) {
  return {
    filename: "src/a.js",
    options: [],
    report() {},
    sourceCode: {
      getScope: () => ({
        set: { get: () => variable ?? undefined },
        variables: variable ? [variable] : [],
        upper: null,
      }),
    },
  };
}

describe("resolveCursorContractOptions", () => {
  const base = { modules: ["@db/cursors"], executors: ["runCursor"] };

  it("returns null when required fields are missing or invalid", () => {
    assert.equal(resolveCursorContractOptions(undefined), null);
    assert.equal(resolveCursorContractOptions(null), null);
    assert.equal(resolveCursorContractOptions([]), null);
    assert.equal(resolveCursorContractOptions({}), null);
    assert.equal(resolveCursorContractOptions({ modules: ["@db/cursors"] }), null);
    assert.equal(resolveCursorContractOptions({ executors: ["runCursor"] }), null);
    assert.equal(resolveCursorContractOptions({ modules: [], executors: ["runCursor"] }), null);
    assert.equal(resolveCursorContractOptions({ modules: ["@db/cursors"], executors: [] }), null);
    assert.equal(resolveCursorContractOptions({ modules: "x", executors: ["runCursor"] }), null);
    assert.equal(resolveCursorContractOptions({ ...base, include: [1] }), null);
    assert.equal(resolveCursorContractOptions({ ...base, exclude: {} }), null);
    assert.equal(resolveCursorContractOptions({ ...base, includeFiles: [1] }), null);
    assert.equal(resolveCursorContractOptions({ ...base, annotation: 1 }), null);
  });

  it("throws when the annotation is not a valid regular expression", () => {
    assert.throws(
      () => resolveCursorContractOptions({ ...base, annotation: "[" }),
      /annotation is not a valid regular expression/,
    );
  });

  it("applies defaults and compiles the annotation regex", () => {
    const resolved = resolveCursorContractOptions({ ...base, includeFiles: ["./lib/seed.js"] });
    assert.deepEqual(resolved.include, DEFAULT_CURSOR_INCLUDE);
    assert.deepEqual(resolved.exclude, []);
    assert.deepEqual(resolved.includeFiles, ["lib/seed.js"]);
    assert.equal(resolved.annotation.test("/* rows */ SELECT 1"), true);
    assert.equal(resolved.annotation.test("SELECT 1"), false);
    assert.deepEqual(
      resolveCursorContractOptions({ ...base, include: [], exclude: ["**/*.test.js"] }).include,
      [],
    );
    assert.equal(
      resolveCursorContractOptions({ ...base, annotation: "^ok" }).annotation.test("ok"),
      true,
    );
  });
});

describe("matchesCursorFile", () => {
  const resolved = resolveCursorContractOptions({
    ...options,
    exclude: ["**/*.test.js", "**/test-helpers/**"],
    includeFiles: ["lib/test-helpers/seed.js"],
  });

  it("includes exact allowlisted paths even when exclude matches", () => {
    const ctx = (filename, cwd = "/repo") => ({
      filename,
      cwd,
      options: [],
      report() {},
      sourceCode: { getScope: () => ({ upper: null }) },
    });
    assert.equal(matchesCursorFile(ctx("/repo/lib/test-helpers/seed.js"), resolved), true);
    assert.equal(matchesCursorFile(ctx("/repo/src/service.test.js"), resolved), false);
    assert.equal(matchesCursorFile(ctx("/repo/src/service.js"), resolved), true);
    assert.equal(matchesCursorFile(ctx("/repo/README.md"), resolved), false);
    assert.equal(
      matchesCursorFile(ctx("/repo/service.mts"), resolveCursorContractOptions(options)),
      true,
    );
    assert.equal(
      matchesCursorFile(
        {
          filename: "service.js",
          options: [],
          report() {},
          sourceCode: { getScope: () => ({ upper: null }) },
        },
        resolveCursorContractOptions(options),
      ),
      true,
    );
    assert.equal(
      matchesCursorFile(ctx("/repo/service.cjs"), resolveCursorContractOptions(options)),
      false,
    );
    assert.equal(
      matchesCursorFile(
        ctx("/repo/src/a.js"),
        resolveCursorContractOptions({ ...options, include: [] }),
      ),
      false,
    );
    assert.equal(
      matchesCursorFile(
        ctx("/repo/src/a.js"),
        resolveCursorContractOptions({ ...options, include: ["src/a.{js"] }),
      ),
      false,
    );
  });
});

describe("unwrap and findVariable", () => {
  it("returns nullish nodes unchanged and peels TypeScript wrappers", () => {
    assert.equal(unwrap(null), null);
    assert.equal(unwrap(undefined), undefined);
    const inner = identifier("runCursor");
    assert.equal(
      unwrap({
        type: "TSAsExpression",
        expression: { type: "TSNonNullExpression", expression: inner },
      }),
      inner,
    );
    assert.deepEqual(unwrap(identifier("keep")), identifier("keep"));
    assert.deepEqual(
      unwrap({ type: "ChainExpression", expression: identifier("x") }),
      identifier("x"),
    );
    assert.deepEqual(
      unwrap({ type: "TSSatisfiesExpression", expression: identifier("x") }),
      identifier("x"),
    );
    assert.deepEqual(
      unwrap({ type: "TSTypeAssertion", expression: identifier("x") }),
      identifier("x"),
    );
  });

  it("walks scopes and ignores non-identifiers", () => {
    const local = { name: "local", defs: [], references: [] };
    const outer = { name: "outer", defs: [], references: [] };
    const context = {
      filename: "src/a.js",
      options: [],
      report() {},
      sourceCode: {
        getScope: () => ({
          variables: [],
          upper: {
            set: { get: (name) => (name === "outer" ? outer : undefined) },
            variables: [outer],
            upper: null,
          },
        }),
      },
    };
    assert.equal(findVariable(context, { type: "Literal", value: 1 }), null);
    assert.equal(findVariable(context, identifier("missing")), null);
    assert.equal(findVariable(context, identifier("outer")), outer);
    assert.equal(
      findVariable(
        {
          filename: "src/a.js",
          options: [],
          report() {},
          sourceCode: { getScope: () => ({ variables: [local], upper: null }) },
        },
        identifier("local"),
      ),
      local,
    );
    assert.equal(
      findVariable(
        {
          filename: "src/a.js",
          options: [],
          report() {},
          sourceCode: {
            getScope: () => ({ set: { get: () => undefined }, variables: [local], upper: null }),
          },
        },
        identifier("local"),
      ),
      local,
    );
  });
});

describe("propertyName and normalizeFilename", () => {
  it("reads static member names and ignores dynamic ones", () => {
    assert.equal(propertyName(identifier("x")), null);
    assert.equal(
      propertyName({ type: "MemberExpression", computed: false, property: identifier("append") }),
      "append",
    );
    assert.equal(
      propertyName({
        type: "MemberExpression",
        computed: false,
        property: { type: "Literal", value: "append" },
      }),
      null,
    );
    assert.equal(
      propertyName({
        type: "MemberExpression",
        computed: true,
        property: { type: "Literal", value: "runCursor" },
      }),
      "runCursor",
    );
    assert.equal(
      propertyName({
        type: "MemberExpression",
        computed: true,
        property: { type: "Literal", value: 1 },
      }),
      1,
    );
    assert.equal(
      propertyName({
        type: "MemberExpression",
        computed: true,
        property: { type: "Literal", value: true },
      }),
      true,
    );
    assert.equal(
      propertyName({
        type: "MemberExpression",
        computed: true,
        property: { type: "Literal", value: 1n },
      }),
      1n,
    );
    assert.equal(
      propertyName({
        type: "MemberExpression",
        computed: true,
        property: { type: "Literal", value: /x/ },
      }),
      null,
    );
    assert.equal(
      propertyName({
        type: "MemberExpression",
        computed: true,
        property: {
          type: "TemplateLiteral",
          expressions: [],
          quasis: [{ value: { cooked: "runCursor", raw: "runCursor" } }],
        },
      }),
      "runCursor",
    );
    assert.equal(
      propertyName({
        type: "MemberExpression",
        computed: true,
        property: {
          type: "TemplateLiteral",
          expressions: [],
          quasis: [{ value: { cooked: null, raw: "rawName" } }],
        },
      }),
      "rawName",
    );
    assert.equal(
      propertyName({
        type: "MemberExpression",
        computed: true,
        property: { type: "TemplateLiteral", expressions: [identifier("x")], quasis: [] },
      }),
      null,
    );
    assert.equal(
      propertyName({
        type: "MemberExpression",
        computed: true,
        property: identifier("method"),
      }),
      null,
    );
    assert.equal(
      propertyName({
        type: "MemberExpression",
        computed: true,
        property: { type: "TemplateLiteral", expressions: [], quasis: [] },
      }),
      null,
    );
  });

  it("relativizes paths and strips a cwd prefix", () => {
    assert.equal(
      normalizeFilename({ filename: "C:\\repo\\src\\a.ts", cwd: "C:\\repo" }),
      "src/a.ts",
    );
    assert.equal(normalizeFilename({ filename: "/repo/src/a.ts", cwd: "/repo/" }), "src/a.ts");
    assert.equal(normalizeFilename({ filename: "/other/a.ts", cwd: "/repo" }), "/other/a.ts");
    assert.equal(normalizeFilename({ filename: "src/a.ts" }), "src/a.ts");
  });
});

describe("cursor import detection", () => {
  it("accepts named and namespace imports and rejects type-only bindings", () => {
    assert.equal(isCursorModule("@db/cursors", config), true);
    assert.equal(isCursorModule(1, config), false);
    assert.equal(isCursorExecutor("runCursor", config), true);
    assert.equal(isCursorExecutor(1, config), false);
    const named = importVariable("runCursor", "ImportSpecifier");
    assert.equal(
      namedCursorImport(contextFor(named), identifier("runCursor"), helpers, config),
      "runCursor",
    );
    assert.equal(
      namedCursorImport(
        contextFor(named),
        { type: "Literal", value: "runCursor" },
        helpers,
        config,
      ),
      null,
    );
    assert.equal(
      namedCursorImport(
        contextFor(importVariable("runCursor", "ImportSpecifier", { importKind: "type" })),
        identifier("runCursor"),
        helpers,
        config,
      ),
      null,
    );
    assert.equal(
      namedCursorImport(
        contextFor(importVariable("runCursor", "ImportSpecifier", { declarationKind: "type" })),
        identifier("runCursor"),
        helpers,
        config,
      ),
      null,
    );
    const namespace = importVariable("db", "ImportNamespaceSpecifier");
    const object = identifier("db");
    const member = {
      type: "MemberExpression",
      object,
      property: identifier("runCursor"),
      computed: false,
    };
    assert.equal(namespaceCursorImport(contextFor(namespace), object, helpers, config), true);
    assert.equal(namespaceImportMember(contextFor(namespace), member, helpers, config), member);
    assert.equal(
      namespaceCursorMember(contextFor(namespace), member, helpers, config),
      "runCursor",
    );
    assert.equal(namespaceImportMember(contextFor(namespace), object, helpers, config), null);
    assert.equal(
      namespaceImportMember(
        contextFor(namespace),
        { type: "MemberExpression", object: { type: "Literal", value: 1 }, computed: false },
        helpers,
        config,
      ),
      null,
    );
  });
});
