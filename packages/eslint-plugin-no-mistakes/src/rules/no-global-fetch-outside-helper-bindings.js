"use strict";

const LOCAL_BINDING_TYPES = new Set([
  "Variable",
  "Parameter",
  "CatchClause",
  "FunctionName",
  "ClassName",
]);

function variableFromScope(scope, name) {
  const get = scope?.set?.get;
  if (typeof get === "function") return get.call(scope.set, name) ?? null;
  return scope?.variables?.find((item) => item.name === name) ?? null;
}

function resolveVariable(node, context) {
  if (node?.type !== "Identifier") return null;
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = variableFromScope(scope, node.name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function hasLocalBinding(node, context) {
  const variable = resolveVariable(node, context);
  return Boolean(variable?.defs?.some(isRuntimeBinding));
}

function isRuntimeBinding(def) {
  if (LOCAL_BINDING_TYPES.has(def.type)) return !isAmbientDeclaration(def);
  if (def.type !== "ImportBinding") return false;
  return def.node?.importKind !== "type" && def.parent?.importKind !== "type";
}

function isAmbientDeclaration(def) {
  return def.node?.declare === true || def.parent?.declare === true;
}

module.exports = {
  hasLocalBinding,
  isAmbientDeclaration,
  isRuntimeBinding,
  resolveVariable,
  variableFromScope,
};
