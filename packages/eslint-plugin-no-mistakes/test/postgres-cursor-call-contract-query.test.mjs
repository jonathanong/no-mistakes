import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { require } from "./helpers.mjs";

const { findVariable, propertyName, unwrap } = require("../src/rules/postgres-cursor-ast");
const { isDiscardedSqlStatementAppend } = require("../src/rules/postgres-cursor-append");
const {
  directCallParent,
  firstQuasiText,
  isTypeQuery,
  queryHead,
  transparentParent,
} = require("../src/rules/postgres-cursor-query");

const helpers = { findVariable, propertyName, unwrap };
const config = {
  modules: new Set(["@db/cursors"]),
  executors: new Set(["runCursor"]),
  sqlTagModules: new Set(["sql-template-strings"]),
};

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

describe("queryHead and wrappers", () => {
  it("reads literals, templates, sql tags, and rejects writes", () => {
    const sqlSpec = { type: "ImportDefaultSpecifier", importKind: "value" };
    const sqlDecl = {
      type: "ImportDeclaration",
      importKind: "value",
      source: { type: "Literal", value: "sql-template-strings" },
    };
    sqlSpec.parent = sqlDecl;
    const sqlVar = {
      name: "sql",
      defs: [{ type: "ImportBinding", node: sqlSpec, parent: sqlDecl }],
      references: [{ identifier: identifier("sql"), isWrite: () => false }],
    };
    const tagged = {
      type: "TaggedTemplateExpression",
      tag: identifier("sql"),
      quasi: {
        type: "TemplateLiteral",
        quasis: [{ value: { cooked: "/* rows */ SELECT 1", raw: "/* rows */ SELECT 1" } }],
      },
    };
    assert.equal(queryHead(contextFor(sqlVar), tagged, helpers, config), "/* rows */ SELECT 1");
    assert.equal(
      queryHead(contextFor(sqlVar), tagged, helpers, {
        ...config,
        sqlTagModules: new Set(["@db/sql"]),
      }),
      null,
    );
    sqlVar.references = [{ identifier: identifier("sql"), isWrite: () => true }];
    assert.equal(queryHead(contextFor(sqlVar), tagged, helpers, config), null);
    sqlVar.references = [{ identifier: identifier("sql"), isWrite: () => false }];
    assert.equal(
      queryHead(
        contextFor(sqlVar),
        {
          type: "TaggedTemplateExpression",
          tag: identifier("sql"),
          quasi: {
            type: "TemplateLiteral",
            quasis: [{ value: { cooked: null, raw: "SELECT \\8" } }],
          },
        },
        helpers,
        config,
      ),
      null,
    );
    const taggedSql = { type: "TaggedTemplateExpression", tag: identifier("sql") };
    assert.equal(
      queryHead(
        contextFor(
          importVariable("sql", "ImportDefaultSpecifier", {
            importKind: "type",
            module: "sql-template-strings",
          }),
        ),
        taggedSql,
        helpers,
        config,
      ),
      null,
    );
    assert.equal(
      queryHead(
        contextFor(
          importVariable("sql", "ImportDefaultSpecifier", {
            declarationKind: "type",
            module: "sql-template-strings",
          }),
        ),
        taggedSql,
        helpers,
        config,
      ),
      null,
    );
    assert.equal(
      queryHead(
        contextFor({
          name: "sql",
          defs: [
            {
              type: "ImportBinding",
              node: { type: "ImportDefaultSpecifier", importKind: "value" },
            },
          ],
          references: [],
        }),
        taggedSql,
        helpers,
        config,
      ),
      null,
    );
    assert.equal(
      queryHead(
        contextFor(null),
        { type: "Literal", value: "/* rows */ SELECT 1" },
        helpers,
        config,
      ),
      "/* rows */ SELECT 1",
    );
    assert.equal(queryHead(contextFor(null), { type: "Literal", value: 1 }, helpers, config), null);
    assert.equal(
      queryHead(
        contextFor(null),
        { type: "TemplateLiteral", quasis: [{ value: { cooked: null, raw: "SELECT 1" } }] },
        helpers,
        config,
      ),
      "SELECT 1",
    );
    assert.equal(
      queryHead(
        contextFor(null),
        { type: "TemplateLiteral", quasis: [{ value: {} }] },
        helpers,
        config,
      ),
      null,
    );
    assert.equal(
      queryHead(
        contextFor(null),
        { type: "TaggedTemplateExpression", tag: { type: "Literal", value: "sql" } },
        helpers,
        config,
      ),
      null,
    );
  });

  it("allows discarded appends and type-query references on a const binding", () => {
    const init = { type: "Literal", value: "/* rows */ SELECT 1" };
    const id = identifier("statement");
    const declarator = { type: "VariableDeclarator", id, init };
    const declaration = { type: "VariableDeclaration", kind: "const" };
    declarator.parent = declaration;
    const arg = identifier("statement");
    const callee = identifier("runCursor");
    const call = { type: "CallExpression", callee, arguments: [arg] };
    arg.parent = call;
    const importVar = importVariable("runCursor", "ImportSpecifier");
    const statementVar = {
      name: "statement",
      defs: [{ type: "Variable", node: declarator }],
      references: [
        { identifier: id, isWrite: () => true },
        { identifier: arg, isWrite: () => false },
      ],
    };
    const mixed = {
      filename: "src/a.js",
      options: [],
      report() {},
      sourceCode: {
        getScope: () => ({
          set: {
            get: (name) =>
              name === "runCursor" ? importVar : name === "statement" ? statementVar : undefined,
          },
          variables: [importVar, statementVar],
          upper: null,
        }),
      },
    };
    assert.equal(queryHead(mixed, arg, helpers, config), "/* rows */ SELECT 1");
    const typeId = identifier("statement");
    const typeQuery = { type: "TSTypeQuery", exprName: typeId };
    typeId.parent = typeQuery;
    statementVar.references.push({ identifier: typeId, isWrite: () => false });
    assert.equal(queryHead(mixed, arg, helpers, config), "/* rows */ SELECT 1");
    assert.equal(queryHead(mixed, identifier("missing"), helpers, config), null);
    assert.equal(
      queryHead(
        contextFor(importVariable("runCursor", "ImportSpecifier")),
        identifier("runCursor"),
        helpers,
        config,
      ),
      null,
    );
    assert.equal(
      queryHead(
        contextFor({
          name: "statement",
          defs: [
            { type: "Variable", node: declarator },
            { type: "Variable", node: declarator },
          ],
          references: [],
        }),
        identifier("statement"),
        helpers,
        config,
      ),
      null,
    );
    assert.equal(
      queryHead(
        contextFor(null),
        {
          type: "TaggedTemplateExpression",
          tag: identifier("sql"),
          quasi: {
            type: "TemplateLiteral",
            quasis: [{ value: { cooked: null, raw: "SELECT 1" } }],
          },
        },
        helpers,
        config,
      ),
      null,
    );
  });

  it("detects discarded append chains and type queries", () => {
    const object = identifier("query");
    const member = { type: "MemberExpression", object, property: identifier("append") };
    const call = { type: "CallExpression", callee: member, arguments: [] };
    const outerMember = { type: "MemberExpression", object: call, property: identifier("append") };
    const outer = { type: "CallExpression", callee: outerMember, arguments: [] };
    const statement = { type: "ExpressionStatement", expression: outer };
    object.parent = member;
    member.parent = call;
    call.parent = outerMember;
    outerMember.parent = outer;
    outer.parent = statement;
    assert.equal(isDiscardedSqlStatementAppend(object, helpers, transparentParent), true);
    const assigned = { type: "VariableDeclarator", init: call };
    call.parent = assigned;
    assert.equal(isDiscardedSqlStatementAppend(object, helpers, transparentParent), false);
    const inner = identifier("runCursor");
    const asNode = { type: "TSAsExpression", expression: inner };
    inner.parent = asNode;
    const callParent = { type: "CallExpression", callee: asNode };
    asNode.parent = callParent;
    assert.equal(directCallParent(inner), callParent);
    const qualified = { type: "TSQualifiedName", left: inner };
    inner.parent = qualified;
    const query = { type: "TSTypeQuery" };
    qualified.parent = query;
    assert.equal(isTypeQuery(inner), true);
    const computed = identifier("query");
    const computedMember = {
      type: "MemberExpression",
      object: computed,
      property: identifier("append"),
      computed: true,
    };
    computed.parent = computedMember;
    assert.equal(isDiscardedSqlStatementAppend(computed, helpers, transparentParent), false);
    const dangling = identifier("query");
    const danglingMember = {
      type: "MemberExpression",
      object: dangling,
      property: identifier("append"),
      computed: false,
    };
    dangling.parent = danglingMember;
    assert.equal(isDiscardedSqlStatementAppend(dangling, helpers, transparentParent), false);
    danglingMember.parent = { type: "CallExpression", callee: identifier("other") };
    assert.equal(isDiscardedSqlStatementAppend(dangling, helpers, transparentParent), false);
  });
});

describe("firstQuasiText", () => {
  it("returns null when a template has no quasis or only raw text is unavailable", () => {
    assert.equal(firstQuasiText(undefined, true), null);
    assert.equal(firstQuasiText({ type: "TemplateLiteral", quasis: [] }, true), null);
    assert.equal(
      firstQuasiText(
        { type: "TemplateLiteral", quasis: [{ value: { cooked: null, raw: "x" } }] },
        false,
      ),
      null,
    );
    assert.equal(
      firstQuasiText(
        { type: "TemplateLiteral", quasis: [{ value: { cooked: null, raw: "x" } }] },
        true,
      ),
      "x",
    );
  });
});
