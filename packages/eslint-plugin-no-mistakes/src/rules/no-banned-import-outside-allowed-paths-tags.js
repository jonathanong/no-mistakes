"use strict";

const { literalString, memberPropertyName } = require("./module-mock-helpers");
const { hasLocalBinding, resolveVariable } = require("./no-global-fetch-outside-helper-bindings");
const { hasBannedName } = require("./no-banned-import-outside-allowed-paths-config");

// Tag shapes tracked per resolved variable:
//   { kind: "direct", module, name }   - this binding IS a specific banned export's value
//   { kind: "object", modules: Set }   - this binding is "the module object" (namespace/default
//                                        import, require()/dynamic-import() result, or a spread
//                                        merge of such objects); member access and destructure
//                                        against it are checked against each module's banned names
//   { kind: "require-fn" }             - this binding is a Node `require`-shaped function, either
//                                        the global `require` or the result of calling createRequire()
//   { kind: "create-require" }         - this binding is Node's `createRequire` itself

const UNWRAP_TYPES = new Set([
  "ChainExpression",
  "TSAsExpression",
  "TSSatisfiesExpression",
  "TSNonNullExpression",
  "TSInstantiationExpression",
  "TSTypeAssertion",
]);

function unwrapExpression(node) {
  let current = node;
  while (current && (current.type === "AwaitExpression" || UNWRAP_TYPES.has(current.type))) {
    current = current.type === "AwaitExpression" ? current.argument : current.expression;
  }
  return current;
}

function isUnshadowedRequire(node, context) {
  return node?.type === "Identifier" && node.name === "require" && !hasLocalBinding(node, context);
}

function tagForIdentifier(node, context, aliasMap) {
  if (node.type !== "Identifier") return null;
  const variable = resolveVariable(node, context);
  return (variable && aliasMap.get(variable)) ?? null;
}

function tagForCallExpression(node, context, aliasMap, config) {
  const calleeTag = tagForIdentifier(node.callee, context, aliasMap);
  if (calleeTag?.kind === "create-require") return { kind: "require-fn" };
  const isRequireFn = calleeTag?.kind === "require-fn";
  if (!isRequireFn && !isUnshadowedRequire(node.callee, context)) return null;
  const specifier = literalString(node.arguments[0]);
  if (!specifier || !config.has(specifier)) return null;
  return { kind: "object", modules: new Set([specifier]) };
}

function tagForMemberExpression(node, context, aliasMap, config) {
  const objectTag = tagForIdentifier(node.object, context, aliasMap);
  if (objectTag?.kind !== "object") return null;
  const name = memberPropertyName(node);
  if (!name) return null;
  for (const module of objectTag.modules) {
    if (hasBannedName(config, module, name)) return { kind: "direct", module, name };
  }
  return null;
}

function tagForDynamicImport(node, config) {
  const specifier = literalString(node.source);
  if (!specifier || !config.has(specifier)) return null;
  return { kind: "object", modules: new Set([specifier]) };
}

function tagForSpread(node, context, aliasMap, config) {
  const modules = new Set();
  for (const property of node.properties) {
    if (property.type !== "SpreadElement") continue;
    const tag = tagForExpression(property.argument, context, aliasMap, config);
    if (tag?.kind === "object") for (const module of tag.modules) modules.add(module);
  }
  return modules.size > 0 ? { kind: "object", modules } : null;
}

function tagForExpression(node, context, aliasMap, config) {
  const unwrapped = unwrapExpression(node);
  if (!unwrapped) return null;
  if (unwrapped.type === "Identifier") return tagForIdentifier(unwrapped, context, aliasMap);
  if (unwrapped.type === "MemberExpression") {
    return tagForMemberExpression(unwrapped, context, aliasMap, config);
  }
  if (unwrapped.type === "CallExpression") {
    return tagForCallExpression(unwrapped, context, aliasMap, config);
  }
  if (unwrapped.type === "ImportExpression") return tagForDynamicImport(unwrapped, config);
  if (unwrapped.type === "ObjectExpression")
    return tagForSpread(unwrapped, context, aliasMap, config);
  return null;
}

module.exports = {
  isUnshadowedRequire,
  tagForExpression,
  tagForIdentifier,
  unwrapExpression,
};
