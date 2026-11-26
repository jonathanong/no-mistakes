"use strict";

const { unwrapSequence } = require("./async-ast");
const { importFact } = require("./import-resolution");
const { literalString } = require("./module-mock-helpers");
const { isGlobalPromiseAlias } = require("./react-no-use-promise-resolve-alias");
const { resolveVariable } = require("./test-no-shared-state-aliases");

function staticName(node, computed) {
  const current = unwrapSequence(node);
  if (!current) return null;
  if (computed && current.type === "Identifier") return null;
  if (!computed && typeof current.name === "string" && current.type !== "Literal")
    return current.name;
  return literalString(current);
}

function variableDef(node, context) {
  const variable = node?.type === "Identifier" ? resolveVariable(node, context) : null;
  return variable?.defs.length === 1 ? variable.defs[0] : null;
}

function patternKey(pattern, name) {
  if (pattern?.type !== "ObjectPattern") return null;
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    let value = property.value;
    while (value?.type === "AssignmentPattern") value = value.left;
    if (value?.type === "Identifier" && value.name === name) {
      return property.computed ? literalString(property.key) : staticName(property.key, false);
    }
  }
  return null;
}

function namespaceStatus(node, context) {
  const fact = importFact(node, context);
  if (!fact) return "no";
  if (fact.unresolved) return "unresolved";
  if (
    !fact.typeOnly &&
    fact.module === "react" &&
    (fact.kind === "namespace" || fact.kind === "default")
  ) {
    return "yes";
  }
  return "no";
}

function isReactNamespace(node, context, seen = new Set()) {
  return namespaceStatus(node, context, seen) === "yes";
}

function namedUse(fact) {
  return Boolean(
    fact &&
    !fact.unresolved &&
    !fact.typeOnly &&
    fact.module === "react" &&
    fact.kind === "named" &&
    fact.imported === "use",
  );
}

function reactUseStatus(node, context, useNames, seen = new Set()) {
  let callee = unwrapSequence(node);
  if (callee?.type === "MemberExpression") {
    const name = staticName(callee.property, callee.computed);
    if (name === "call" || name === "apply") callee = unwrapSequence(callee.object);
  }
  if (callee?.type === "Identifier") {
    if (seen.has(callee.name)) return "no";
    const fact = importFact(callee, context);
    if (namedUse(fact)) return "yes";
    if (fact?.unresolved) return "unresolved";
    if (fact) return "no";
    const def = variableDef(callee, context);
    if (def?.type === "Variable" && def.node?.parent?.kind === "const" && def.node.init) {
      seen.add(callee.name);
      if (def.node.id.type === "Identifier") {
        return reactUseStatus(def.node.init, context, useNames, seen);
      }
      if (patternKey(def.node.id, callee.name) !== "use") return "no";
      return namespaceStatus(def.node.init, context);
    }
    if (def) return "no";
    return useNames.has(callee.name) ? "yes" : "no";
  }
  if (
    callee?.type === "MemberExpression" &&
    staticName(callee.property, callee.computed) === "use"
  ) {
    return namespaceStatus(callee.object, context);
  }
  return "no";
}

function isReactUse(node, context, useNames, seen = new Set()) {
  return reactUseStatus(node, context, useNames, seen) === "yes";
}

function isPromiseResolve(node) {
  const current = unwrapSequence(node);
  if (current?.type !== "CallExpression") return false;
  let callee = unwrapSequence(current.callee);
  if (
    callee?.type === "MemberExpression" &&
    ["call", "apply"].includes(staticName(callee.property, callee.computed))
  ) {
    callee = unwrapSequence(callee.object);
  }
  const object = unwrapSequence(callee?.object);
  return (
    callee?.type === "MemberExpression" &&
    object?.type === "Identifier" &&
    object.name === "Promise" &&
    staticName(callee.property, callee.computed) === "resolve"
  );
}

function isAliasedPromiseResolve(node, context) {
  const current = unwrapSequence(node);
  if (
    current?.type !== "CallExpression" ||
    current.optional ||
    current.arguments.length !== 1 ||
    current.arguments[0].type === "SpreadElement"
  ) {
    return false;
  }
  const callee = unwrapSequence(current.callee);
  const object = unwrapSequence(callee?.object);
  return (
    callee?.type === "MemberExpression" &&
    !callee.optional &&
    object?.type === "Identifier" &&
    staticName(callee.property, callee.computed) === "resolve" &&
    isGlobalPromiseAlias(object, context)
  );
}

function ordinaryUseCall(node) {
  const current = unwrapSequence(node);
  if (
    !current ||
    current.optional ||
    current.arguments.length !== 1 ||
    current.arguments[0].type === "SpreadElement"
  ) {
    return false;
  }
  const callee = unwrapSequence(current.callee);
  return (
    callee?.type === "Identifier" ||
    (callee?.type === "MemberExpression" &&
      !callee.optional &&
      !["call", "apply"].includes(staticName(callee.property, callee.computed)))
  );
}

function forwardedArgument(node) {
  const forwarded = unwrapSequence(node.callee);
  const forwarding =
    forwarded?.type === "MemberExpression" &&
    ["call", "apply"].includes(staticName(forwarded.property, forwarded.computed));
  if (!forwarding) return node.arguments[0];
  if (staticName(forwarded.property, forwarded.computed) !== "apply") return node.arguments[1];
  const list = unwrapSequence(node.arguments[1]);
  return list?.type === "ArrayExpression" ? list.elements?.[0] : undefined;
}

module.exports = {
  forwardedArgument,
  isAliasedPromiseResolve,
  isPromiseResolve,
  isReactNamespace,
  isReactUse,
  ordinaryUseCall,
  patternKey,
  reactUseStatus,
  staticName,
};
