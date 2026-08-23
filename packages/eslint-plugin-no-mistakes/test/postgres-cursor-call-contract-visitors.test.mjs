import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { plugin } from "./helpers.mjs";

const rule = plugin.rules["postgres-cursor-call-contract"];
const options = { modules: ["@db/cursors"], executors: ["runCursor"] };

function importVar(name, specifierType, imported = { name }) {
  const specifier = {
    type: specifierType,
    importKind: "value",
    imported,
    local: { name },
  };
  const declaration = {
    type: "ImportDeclaration",
    importKind: "value",
    source: { type: "Literal", value: "@db/cursors" },
  };
  specifier.parent = declaration;
  const identifier = { type: "Identifier", name };
  return {
    name,
    defs: [{ type: "ImportBinding", node: specifier }],
    references: [{ identifier, isWrite: () => false }],
  };
}

function mockContext(variable, reports, extra = {}) {
  return {
    filename: extra.filename ?? "/repo/src/a.js",
    cwd: extra.cwd ?? "/repo",
    options: extra.options ?? [options],
    report: (descriptor) => reports.push({ messageId: descriptor.messageId }),
    sourceCode: {
      getScope: () => ({
        set: {
          get: (name) => (name && variable && name === variable.name ? variable : undefined),
        },
        variables: variable ? [variable] : [],
        upper: null,
      }),
    },
  };
}

describe("postgres-cursor-call-contract visitors", () => {
  it("skips type-only exports, type queries, and non-references", () => {
    const variable = importVar("runCursor", "ImportSpecifier", { value: "runCursor" });
    const reports = [];
    const visitors = rule.create(mockContext(variable, reports));
    const identifier = variable.references[0].identifier;
    identifier.parent = { type: "ExportSpecifier", exportKind: "type" };
    visitors.Identifier(identifier);
    const parentType = { type: "ExportNamedDeclaration", exportKind: "type" };
    identifier.parent = { type: "ExportSpecifier", exportKind: "value", parent: parentType };
    visitors.Identifier(identifier);
    identifier.parent = { type: "TSTypeQuery" };
    visitors.Identifier(identifier);
    identifier.parent = { type: "VariableDeclarator" };
    variable.references[0] = {
      identifier: { type: "Identifier", name: "runCursor" },
      isWrite: () => false,
    };
    visitors.Identifier(identifier);
    assert.deepEqual(reports, []);
  });

  it("reports namespace members that are not called directly and ignores type queries", () => {
    const variable = importVar("db", "ImportNamespaceSpecifier");
    const reports = [];
    const visitors = rule.create(mockContext(variable, reports));
    const object = { type: "Identifier", name: "db" };
    const member = {
      type: "MemberExpression",
      object,
      property: { type: "Identifier", name: "runCursor" },
      computed: false,
    };
    object.parent = member;
    visitors.MemberExpression(member);
    member.parent = { type: "TSTypeQuery" };
    visitors.MemberExpression(member);
    assert.deepEqual(reports, [{ messageId: "directUse" }]);
  });

  it("does not report type-only or unrelated re-exports", () => {
    const reports = [];
    const visitors = rule.create(mockContext(null, reports));
    visitors.ExportAllDeclaration({
      type: "ExportAllDeclaration",
      exportKind: "type",
      source: { value: "@db/cursors" },
    });
    visitors.ExportNamedDeclaration({
      type: "ExportNamedDeclaration",
      source: { value: "@db/cursors" },
      specifiers: [
        { type: "ExportSpecifier", exportKind: "type", local: { name: "runCursor" } },
        { type: "ExportSpecifier", local: { value: "helper" } },
      ],
    });
    visitors.ExportNamedDeclaration({
      type: "ExportNamedDeclaration",
      source: { value: "@db/cursors" },
    });
    assert.deepEqual(reports, []);
    visitors.ExportNamedDeclaration({
      type: "ExportNamedDeclaration",
      source: { value: "@db/cursors" },
      specifiers: [{ type: "ExportSpecifier", local: { value: "runCursor" } }],
    });
    assert.deepEqual(reports, [{ messageId: "directUse" }]);
  });

  it("returns no visitors when unconfigured or excluded", () => {
    assert.deepEqual(rule.create(mockContext(null, [], { options: [] })), {});
    assert.deepEqual(
      rule.create(
        mockContext(null, [], {
          filename: "/repo/src/a.test.js",
          options: [{ ...options, exclude: ["**/*.test.js"] }],
        }),
      ),
      {},
    );
  });

  it("reports missing and unannotated SQL and ignores unrelated callees", () => {
    const variable = importVar("runCursor", "ImportSpecifier");
    const reports = [];
    const visitors = rule.create(mockContext(variable, reports));
    const callee = variable.references[0].identifier;
    visitors.CallExpression({ type: "CallExpression", callee, arguments: [] });
    visitors.CallExpression({
      type: "CallExpression",
      callee,
      arguments: [{ type: "Literal", value: "SELECT 1" }],
    });
    visitors.CallExpression({
      type: "CallExpression",
      callee,
      arguments: [{ type: "Literal", value: "/* rows */ SELECT 1" }],
    });
    visitors.CallExpression({
      type: "CallExpression",
      callee: { type: "Identifier", name: "other" },
      arguments: [{ type: "Literal", value: "SELECT 1" }],
    });
    assert.deepEqual(reports, [{ messageId: "staticQuery" }, { messageId: "annotation" }]);
  });
});
