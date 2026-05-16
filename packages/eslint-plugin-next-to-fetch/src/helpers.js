"use strict";

function rule(meta, create) {
  return { meta, create };
}

function isStaticString(node) {
  if (!node) return false;
  if (node.type === "Literal") return typeof node.value === "string";
  if (node.type === "TemplateLiteral") return node.expressions.length === 0;
  return false;
}

const LOCAL_BINDING_TYPES = new Set(["Variable", "Parameter", "CatchClause", "FunctionName"]);

function isFetchShadowed(scope) {
  while (scope) {
    const variable = scope.variables.find((v) => v.name === "fetch");
    if (variable) return variable.defs.some((def) => LOCAL_BINDING_TYPES.has(def.type));
    scope = scope.upper;
  }
  return false;
}

function isFetchCall(node, context) {
  if (node.callee.type !== "Identifier" || node.callee.name !== "fetch") return false;
  return !isFetchShadowed(context.sourceCode.getScope(node));
}

module.exports = { isFetchCall, isStaticString, rule };
