"use strict";

const {
  memberPropertyName,
  propertyName,
  repoRelativeFilename,
  stringMatches,
} = require("./module-mock-helpers");

const GLOBAL_FETCH_ROOTS = new Set(["globalThis", "window", "self", "global"]);
const LOCAL_BINDING_TYPES = new Set([
  "Variable",
  "Parameter",
  "CatchClause",
  "FunctionName",
  "ImportBinding",
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
  return Boolean(variable?.defs?.some((def) => LOCAL_BINDING_TYPES.has(def.type)));
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

function childNodes(node) {
  const children = [];
  for (const [key, value] of Object.entries(node)) {
    if (key === "parent") continue;
    if (Array.isArray(value)) {
      for (const item of value) {
        if (item?.type) children.push(item);
      }
    } else if (value?.type) {
      children.push(value);
    }
  }
  return children;
}

function collectVariableDeclarators(node, declarators = []) {
  if (node.type === "VariableDeclarator") declarators.push(node);
  for (const child of childNodes(node)) collectVariableDeclarators(child, declarators);
  return declarators;
}

function collectFetchAliases(program, context, aliases) {
  const declarators = collectVariableDeclarators(program);
  let changed = true;
  while (changed) {
    changed = false;
    for (const node of declarators) {
      if (node.parent?.type !== "VariableDeclaration" || node.parent.kind !== "const") continue;
      const before = aliases.size;
      recordObjectPatternFetchAliases(node.id, node.init, context, aliases);
      if (node.id.type === "Identifier" && isGlobalFetchExpression(node.init, context, aliases)) {
        setAlias(node.id, true, context, aliases);
      }
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
  childNodes,
  collectFetchAliases,
  collectVariableDeclarators,
  hasLocalBinding,
  isGlobalFetchExpression,
  isGlobalFetchMember,
  recordObjectPatternFetchAliases,
  setAlias,
  setObjectPatternFetchAliases,
  shouldCheckFile,
  unwrapTSAndChain,
};
