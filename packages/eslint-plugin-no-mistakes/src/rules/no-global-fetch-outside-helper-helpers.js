"use strict";

const {
  memberPropertyName,
  propertyName,
  repoRelativeFilename,
  stringMatches,
} = require("./module-mock-helpers");
const {
  childNodes,
  collectAssignmentExpressions,
  collectVariableDeclarators,
  isAlwaysExecutedChild,
  isMaybeExecuted,
} = require("./no-global-fetch-outside-helper-traversal");

const GLOBAL_FETCH_ROOTS = new Set(["globalThis", "window", "self", "global"]);
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

function unwrapTSAndChain(node) {
  while (
    node &&
    (node.type === "ChainExpression" ||
      node.type === "TSAsExpression" ||
      node.type === "TSSatisfiesExpression" ||
      node.type === "TSNonNullExpression" ||
      node.type === "TSInstantiationExpression" ||
      node.type === "TSTypeAssertion")
  ) {
    node = node.expression;
  }
  return node;
}

function isUnshadowedGlobalRoot(node, context) {
  const unwrapped = unwrapTSAndChain(node);
  return (
    unwrapped?.type === "Identifier" &&
    GLOBAL_FETCH_ROOTS.has(unwrapped.name) &&
    !hasLocalBinding(unwrapped, context)
  );
}

function isGlobalFetchMember(node, context) {
  const unwrapped = unwrapTSAndChain(node);
  return (
    unwrapped?.type === "MemberExpression" &&
    memberPropertyName(unwrapped) === "fetch" &&
    isUnshadowedGlobalRoot(unwrapped.object, context)
  );
}

function isGlobalFetchExpression(node, context, aliases) {
  const unwrapped = unwrapTSAndChain(node);
  if (!unwrapped) return false;
  if (unwrapped.type === "Identifier") {
    const variable = resolveVariable(unwrapped, context);
    if (variable && aliases.has(variable)) return true;
    if (unwrapped.name === "fetch") return !hasLocalBinding(unwrapped, context);
    return false;
  }
  return isGlobalFetchMember(unwrapped, context);
}

function bindingIdentifier(node) {
  if (node?.type === "Identifier") return node;
  return node?.type === "AssignmentPattern" && node.left.type === "Identifier" ? node.left : null;
}

function setAlias(id, enabled, context, aliases) {
  const variable = resolveVariable(id, context);
  if (!variable) return;
  if (enabled) {
    aliases.add(variable);
  } else {
    aliases.delete(variable);
  }
}

function recordObjectPatternFetchAliases(id, init, context, aliases) {
  setObjectPatternFetchAliases(id, isUnshadowedGlobalRoot(init, context), context, aliases);
}

function setObjectPatternFetchAliases(id, enabled, context, aliases) {
  if (id.type !== "ObjectPattern") return;
  for (const property of id.properties) {
    if (property.type !== "Property" || propertyName(property.key) !== "fetch") continue;
    setAlias(bindingIdentifier(property.value), enabled, context, aliases);
  }
}

function recordVariableFetchAliases(node, context, aliases) {
  if (!node.init) return;
  if (node.id.type === "Identifier") {
    setAlias(node.id, isGlobalFetchExpression(node.init, context, aliases), context, aliases);
    return;
  }
  recordObjectPatternFetchAliases(node.id, node.init, context, aliases);
}

function recordAssignmentFetchAliases(node, context, aliases) {
  if (node.operator !== "=") return;
  if (node.left?.type === "Identifier") {
    setAlias(node.left, isGlobalFetchExpression(node.right, context, aliases), context, aliases);
    return;
  }
  recordObjectPatternFetchAliases(node.left, node.right, context, aliases);
}

function collectPossibleAlias(node, context, aliases) {
  if (node.type === "VariableDeclarator") {
    if (!node.init) return;
    if (node.id.type === "Identifier" && isGlobalFetchExpression(node.init, context, aliases)) {
      setAlias(node.id, true, context, aliases);
      return;
    }
    if (isUnshadowedGlobalRoot(node.init, context))
      recordObjectPatternFetchAliases(node.id, node.init, context, aliases);
    return;
  }
  if (node.operator !== "=") return;
  if (node.left?.type === "Identifier") {
    if (!isGlobalFetchExpression(node.right, context, aliases)) return;
    setAlias(node.left, true, context, aliases);
  } else if (isUnshadowedGlobalRoot(node.right, context)) {
    setObjectPatternFetchAliases(node.left, true, context, aliases);
  }
}

function collectFetchAliases(program, context, aliases) {
  const candidates = [
    ...collectVariableDeclarators(program),
    ...collectAssignmentExpressions(program),
  ];
  let changed = true;
  while (changed) {
    changed = false;
    for (const node of candidates) {
      if (isMaybeExecuted(node)) continue;
      const before = aliases.size;
      collectPossibleAlias(node, context, aliases);
      changed ||= aliases.size > before;
    }
  }
}

function shouldCheckFile(filename, options) {
  const checked = options?.checkedPathPatterns ?? [];
  if (checked.length === 0) return false;
  const file = repoRelativeFilename(filename);
  return stringMatches(file, checked) && !stringMatches(file, options.allowedPathPatterns ?? []);
}

module.exports = {
  bindingIdentifier,
  collectAssignmentExpressions,
  childNodes,
  collectFetchAliases,
  collectVariableDeclarators,
  hasLocalBinding,
  isRuntimeBinding,
  isAmbientDeclaration,
  isGlobalFetchExpression,
  isGlobalFetchMember,
  isAlwaysExecutedChild,
  isMaybeExecuted,
  recordAssignmentFetchAliases,
  recordObjectPatternFetchAliases,
  recordVariableFetchAliases,
  setAlias,
  setObjectPatternFetchAliases,
  shouldCheckFile,
  unwrapTSAndChain,
};
