"use strict";

const { unwrapExpression } = require("./async-ast");

function findVariable(sourceCode, identifier) {
  if (!sourceCode?.getScope) return null;
  let scope = sourceCode.getScope(identifier);
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === identifier.name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function isShadowedUndefined(identifier, sourceCode) {
  const variable = findVariable(sourceCode, identifier);
  return Boolean(variable?.defs.some((def) => def.type && def.type !== "ImplicitGlobalVariable"));
}

function isDefinitelyUndefinedLiteral(node, sourceCode) {
  const current = unwrapExpression(node);
  if (!current) return false;
  if (current.type === "UnaryExpression" && current.operator === "void") return true;
  if (current.type === "Identifier" && current.name === "undefined") {
    return !isShadowedUndefined(current, sourceCode);
  }
  return false;
}

function hasLaterWrite(variable) {
  return variable.references.some((reference) => reference.isWrite() && !reference.init);
}

function isDefinitelyUndefinedBinding(variable, sourceCode, seen) {
  if (variable.defs.length !== 1) return false;
  const def = variable.defs[0];
  if (def.type !== "Variable" || !def.node?.init || hasLaterWrite(variable)) return false;
  return isDefinitelyUndefinedValue(def.node.init, sourceCode, seen);
}

function isDefinitelyUndefinedValue(node, sourceCode, seen = new Set()) {
  if (isDefinitelyUndefinedLiteral(node, sourceCode)) return true;
  const current = unwrapExpression(node);
  if (current?.type !== "Identifier") return false;
  const variable = findVariable(sourceCode, current);
  if (!variable || seen.has(variable)) return false;
  seen.add(variable);
  return isDefinitelyUndefinedBinding(variable, sourceCode, seen);
}

module.exports = {
  isDefinitelyUndefinedValue,
};
