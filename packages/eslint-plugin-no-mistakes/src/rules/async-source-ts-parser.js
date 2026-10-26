"use strict";
const { expression } = require("./async-source-ts-expressions");
const { declaration } = require("./async-source-ts-declarations");
const { statement } = require("./async-source-ts-statements");
const { sourceScopes } = require("./async-source-ts-scopes");

// This is a proof input adapter, not a TypeScript compiler or a general parser.
// Reject the whole implementation if any syntax cannot be represented faithfully.
function parseTypeScript(ts, text, filename) {
  if (typeof text !== "string") throw new Error("Missing implementation");
  const source = ts.createSourceFile(filename, text, ts.ScriptTarget.Latest, true);
  if (source.parseDiagnostics.length) throw new Error("Invalid implementation");
  const visitorKeys = {};
  const api = { ts, convert, make, list, annotation, modifiers, property, fn };
  function make(node, type, fields = {}) {
    const range = [node.getStart(source), node.end];
    const start = source.getLineAndCharacterOfPosition(range[0]);
    const end = source.getLineAndCharacterOfPosition(range[1]);
    const result = {
      type,
      range,
      loc: {
        start: { line: start.line + 1, column: start.character },
        end: { line: end.line + 1, column: end.character },
      },
      ...fields,
    };
    const keys = Object.keys(fields).filter((key) => {
      const value = fields[key];
      return value?.type || (Array.isArray(value) && value.some((item) => item?.type));
    });
    visitorKeys[type] = [...new Set([...(visitorKeys[type] || []), ...keys])];
    return result;
  }
  function list(nodes) {
    return nodes ? Array.from(nodes, convert) : [];
  }
  function annotation(node) {
    return node && make(node, "TSTypeAnnotation", { typeAnnotation: convert(node) });
  }
  function modifiers(node) {
    const allowed = [
      ts.SyntaxKind.ExportKeyword,
      ts.SyntaxKind.DefaultKeyword,
      ts.SyntaxKind.AsyncKeyword,
      ts.SyntaxKind.DeclareKeyword,
      ts.SyntaxKind.ReadonlyKeyword,
    ];
    if (node.modifiers?.some((modifier) => !allowed.includes(modifier.kind)))
      throw new Error("Unsupported implementation modifier");
    return (kind) => Boolean(node.modifiers?.some((item) => item.kind === kind));
  }
  function property(node) {
    const computed = ts.isComputedPropertyName(node.name);
    return { key: convert(computed ? node.name.expression : node.name), computed };
  }
  function fn(node, type) {
    const has = modifiers(node);
    return make(node, type, {
      id:
        ts.isFunctionDeclaration(node) || ts.isFunctionExpression(node) ? convert(node.name) : null,
      params: list(node.parameters),
      body: convert(node.body),
      async: has(ts.SyntaxKind.AsyncKeyword),
      generator: Boolean(node.asteriskToken),
      returnType: annotation(node.type),
      typeParameters:
        node.typeParameters &&
        make(node, "TSTypeParameterDeclaration", {
          params: list(node.typeParameters),
        }),
    });
  }
  function convert(node) {
    if (!node) return null;
    if (ts.isParenthesizedExpression(node)) return convert(node.expression);
    const result = declaration(node, api) || statement(node, api) || expression(node, api);
    if (!result) throw new Error(`Unsupported implementation syntax: ${ts.SyntaxKind[node.kind]}`);
    return result;
  }
  const ast = make(source, "Program", { body: list(source.statements), sourceType: "module" });
  function parents(node, parent) {
    node.parent = parent;
    for (const key of visitorKeys[node.type] || []) {
      const value = node[key];
      if (Array.isArray(value)) value.forEach((child) => child && parents(child, node));
      else if (value) parents(value, node);
    }
  }
  parents(ast, null);
  return { ast, visitorKeys, ...sourceScopes(ast, visitorKeys) };
}

module.exports = { parseTypeScript };
