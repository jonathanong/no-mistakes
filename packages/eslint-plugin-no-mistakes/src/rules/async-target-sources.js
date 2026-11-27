"use strict";

const { unwrapExpression } = require("./async-ast");
const { isLocalRequire } = require("./async-target-bindings");
const { literalString } = require("./module-mock-helpers");

function importSpecifierName(specifier) {
  const imported = specifier.imported;
  return imported.type === "Literal" ? String(imported.value) : imported.name;
}

function requireBinding(node, context) {
  const expression = unwrapExpression(node);
  if (expression?.type !== "CallExpression") return null;
  const callee = unwrapExpression(expression.callee);
  if (
    callee?.type !== "Identifier" ||
    callee.name !== "require" ||
    isLocalRequire(callee, context)
  ) {
    return null;
  }
  const source = literalString(unwrapExpression(expression.arguments?.[0]));
  return source === null ? { status: "unresolved" } : { status: "resolved", source };
}

function requireSource(node, context) {
  const binding = requireBinding(node, context);
  return binding?.status === "resolved" ? binding.source : null;
}

module.exports = { importSpecifierName, requireBinding, requireSource };
