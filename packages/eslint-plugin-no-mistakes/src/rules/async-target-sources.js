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
  if (expression?.type !== "CallExpression") return null;
  const callee = unwrapExpression(expression.callee);
  const source = literalString(unwrapExpression(expression.arguments?.[0]));
  if (
    callee?.type !== "Identifier" ||
    callee.name !== "require" ||
    source === null ||
    isLocalRequire(callee, context)
  ) {
    return null;
  }
  return source;
}

module.exports = { importSpecifierName, requireSource };
