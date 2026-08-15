"use strict";

const { propertyName } = require("./module-mock-helpers");
const { bindingIdentifier, resolveVariable } = require("./no-global-fetch-outside-helper-bindings");
const {
  collectAssignmentExpressions,
  collectVariableDeclarators,
  isMaybeExecuted,
  isOptionalChainArgument,
} = require("./no-global-fetch-outside-helper-traversal");
const { hasBannedName } = require("./no-banned-import-outside-allowed-paths-config");
const { tagForExpression } = require("./no-banned-import-outside-allowed-paths-tags");

function setOrClearTag(identifier, tag, context, aliasMap, clearedAliases) {
  const variable = resolveVariable(identifier, context);
  if (!variable) return;
  if (tag) {
    aliasMap.set(variable, tag);
    clearedAliases?.delete(variable);
  } else {
    aliasMap.delete(variable);
    clearedAliases?.add(variable);
  }
}

function resolvePropertyTag(initTag, key, config) {
  if (initTag?.kind !== "object" || !key) return null;
  for (const module of initTag.modules) {
    if (hasBannedName(config, module, key)) return { kind: "direct", module, name: key };
  }
  return null;
}

// Real-time (control-flow-sensitive) destructure recorder: clears every
// destructured identifier that doesn't resolve to a banned name, and
// conservatively keeps the whole object tag on a rest element.
function applyObjectPatternTag(pattern, initTag, context, aliasMap, clearedAliases, config) {
  for (const property of pattern.properties) {
    if (property.type === "RestElement") {
      const identifier = bindingIdentifier(property.argument);
      const tag = initTag?.kind === "object" ? initTag : null;
      if (identifier) setOrClearTag(identifier, tag, context, aliasMap, clearedAliases);
      continue;
    }
    if (property.type !== "Property") continue;
    const identifier = bindingIdentifier(property.value);
    if (!identifier) continue;
    const tag = resolvePropertyTag(initTag, propertyName(property.key), config);
    setOrClearTag(identifier, tag, context, aliasMap, clearedAliases);
  }
}

function recordVariableTag(
  node,
  context,
  aliasMap,
  clearedAliases,
  config,
  readAliasMap = aliasMap,
) {
  if (!node.init) return;
  if (node.id.type === "Identifier") {
    const tag = tagForExpression(node.init, context, readAliasMap, config);
    setOrClearTag(node.id, tag, context, aliasMap, clearedAliases);
    return;
  }
  if (node.id.type === "ObjectPattern") {
    const initTag = tagForExpression(node.init, context, readAliasMap, config);
    applyObjectPatternTag(node.id, initTag, context, aliasMap, clearedAliases, config);
  }
}

function recordAssignmentTag(
  node,
  context,
  aliasMap,
  clearedAliases,
  config,
  readAliasMap = aliasMap,
) {
  if (isOptionalChainArgument(node)) return;
  if (node.operator === "||=" || node.operator === "??=") return;
  if (node.operator !== "=") {
    if (node.left?.type === "Identifier") {
      setOrClearTag(node.left, null, context, aliasMap, clearedAliases);
    }
    return;
  }
  if (node.left?.type === "Identifier") {
    const tag = tagForExpression(node.right, context, readAliasMap, config);
    setOrClearTag(node.left, tag, context, aliasMap, clearedAliases);
    return;
  }
  if (node.left?.type === "ObjectPattern") {
    const initTag = tagForExpression(node.right, context, readAliasMap, config);
    applyObjectPatternTag(node.left, initTag, context, aliasMap, clearedAliases, config);
  }
}

// Fixed-point (forward-reference) seeder: add-only, never clears, so repeated
// passes over the whole program monotonically converge (mirrors the
// reference rule's `collectPossibleAlias`).
function applyObjectPatternTagAddOnly(pattern, initTag, context, aliasMap, config) {
  for (const property of pattern.properties) {
    if (property.type === "RestElement") {
      if (initTag.kind !== "object") continue;
      const identifier = bindingIdentifier(property.argument);
      if (identifier) setOrClearTag(identifier, initTag, context, aliasMap);
      continue;
    }
    if (property.type !== "Property") continue;
    const identifier = bindingIdentifier(property.value);
    if (!identifier) continue;
    const tag = resolvePropertyTag(initTag, propertyName(property.key), config);
    if (tag) setOrClearTag(identifier, tag, context, aliasMap);
  }
}

function collectPossibleTag(node, context, aliasMap, config) {
  if (node.type === "VariableDeclarator") {
    if (!node.init) return;
    if (node.id.type === "Identifier") {
      const tag = tagForExpression(node.init, context, aliasMap, config);
      if (tag) setOrClearTag(node.id, tag, context, aliasMap);
      return;
    }
    if (node.id.type === "ObjectPattern") {
      const initTag = tagForExpression(node.init, context, aliasMap, config);
      if (initTag) applyObjectPatternTagAddOnly(node.id, initTag, context, aliasMap, config);
    }
    return;
  }
  if (node.operator !== "=") return;
  if (node.left?.type === "Identifier") {
    const tag = tagForExpression(node.right, context, aliasMap, config);
    if (tag) setOrClearTag(node.left, tag, context, aliasMap);
    return;
  }
  if (node.left?.type === "ObjectPattern") {
    const initTag = tagForExpression(node.right, context, aliasMap, config);
    if (initTag) applyObjectPatternTagAddOnly(node.left, initTag, context, aliasMap, config);
  }
}

function collectBannedAliases(program, context, aliasMap, config) {
  const candidates = [
    ...collectVariableDeclarators(program),
    ...collectAssignmentExpressions(program),
  ];
  let changed = true;
  while (changed) {
    changed = false;
    for (const node of candidates) {
      if (isMaybeExecuted(node)) continue;
      const before = aliasMap.size;
      collectPossibleTag(node, context, aliasMap, config);
      changed ||= aliasMap.size > before;
    }
  }
}

module.exports = {
  collectBannedAliases,
  recordAssignmentTag,
  recordVariableTag,
  setOrClearTag,
};
