"use strict";

const { unwrapExpression } = require("./async-ast");
const { isLocalRequire } = require("./async-target-bindings");
const { literalString } = require("./logical-call");

function importSpecifierName(specifier) {
  const imported = specifier.imported;
  return imported.type === "Literal" ? String(imported.value) : imported.name;
}

function requireCallee(node, context) {
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
  return expression;
}

function requireBinding(node, context) {
  const expression = requireCallee(node, context);
  if (!expression) return null;
  const source = literalString(unwrapExpression(expression.arguments?.[0]));
  return source === null ? { status: "unresolved" } : { status: "resolved", source };
}

function requireSource(node, context) {
  const binding = requireBinding(node, context);
  return binding?.status === "resolved" ? binding.source : null;
}

// `null` is not a require. `{ source: null }` is a require whose specifier is not static.
function requireResolution(node, context) {
  const expression = requireCallee(node, context);
  if (!expression) return null;
  const source = literalString(unwrapExpression(expression.arguments?.[0]));
  return source === null ? { source: null } : { source };
}

module.exports = { importSpecifierName, requireBinding, requireResolution, requireSource };
