"use strict";

const {
  memberPropertyName,
  propertyName,
  repoRelativeFilename,
  stringMatches,
} = require("./module-mock-helpers");
const {
  hasLocalBinding,
  isAmbientDeclaration,
  isRuntimeBinding,
  resolveVariable,
} = require("./no-global-fetch-outside-helper-bindings");
const {
  childNodes,
  collectAssignmentExpressions,
  collectVariableDeclarators,
  isAlwaysExecutedChild,
  isMaybeExecuted,
} = require("./no-global-fetch-outside-helper-traversal");

const GLOBAL_FETCH_ROOTS = new Set(["globalThis", "window", "self", "global"]);

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

function setAlias(id, enabled, context, aliases, clearedAliases) {
  const variable = resolveVariable(id, context);
  if (!variable) return;
  if (enabled) {
    aliases.add(variable);
    clearedAliases?.delete(variable);
  } else {
    aliases.delete(variable);
    clearedAliases?.add(variable);
  }
}

function recordObjectPatternFetchAliases(id, init, context, aliases, clearedAliases) {
  setObjectPatternFetchAliases(
    id,
    isUnshadowedGlobalRoot(init, context),
    context,
    aliases,
    clearedAliases,
  );
}

function setObjectPatternFetchAliases(id, enabled, context, aliases, clearedAliases) {
  if (id.type !== "ObjectPattern") return;
  for (const property of id.properties) {
    if (property.type !== "Property" || propertyName(property.key) !== "fetch") continue;
    setAlias(bindingIdentifier(property.value), enabled, context, aliases, clearedAliases);
  }
}

function recordVariableFetchAliases(node, context, aliases, clearedAliases) {
  if (!node.init) return;
  if (node.id.type === "Identifier") {
    setAlias(
      node.id,
      isGlobalFetchExpression(node.init, context, aliases),
      context,
      aliases,
      clearedAliases,
    );
    return;
  }
  recordObjectPatternFetchAliases(node.id, node.init, context, aliases, clearedAliases);
}

function recordAssignmentFetchAliases(node, context, aliases, clearedAliases) {
  if (node.operator !== "=") {
    if (node.left?.type === "Identifier")
      setAlias(node.left, false, context, aliases, clearedAliases);
    return;
  }
  if (node.left?.type === "Identifier") {
    setAlias(
      node.left,
      isGlobalFetchExpression(node.right, context, aliases),
      context,
      aliases,
      clearedAliases,
    );
    return;
  }
  recordObjectPatternFetchAliases(node.left, node.right, context, aliases, clearedAliases);
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
