"use strict";

const { unwrapExpression } = require("./async-ast");
const { isLocalRequire } = require("./async-target-bindings");
const { literalString } = require("./module-mock-helpers");

function importSpecifierName(specifier) {
  const imported = specifier.imported;
  return imported.type === "Literal" ? String(imported.value) : imported.name;
}

function requireSource(node, context) {
  const expression = unwrapExpression(node);
  const source = literalString(unwrapExpression(expression?.arguments?.[0]));
  if (
    expression?.type !== "CallExpression" ||
    expression.callee.type !== "Identifier" ||
    expression.callee.name !== "require" ||
    source === null ||
    isLocalRequire(expression.callee, context)
  ) {
    return null;
  }
  return source;
}

module.exports = { importSpecifierName, requireSource };
