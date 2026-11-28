"use strict";

const { unwrapSequence } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { importFact } = require("./import-resolution");
const { literalString } = require("./logical-call");
const { isGlobalPromiseAlias } = require("./react-no-use-promise-resolve-alias");

function staticName(node, computed) {
  const current = unwrapSequence(node);
  if (!current) return null;
  if (computed && current.type === "Identifier") return null;
  if (!computed && typeof current.name === "string" && current.type !== "Literal")
    return current.name;
  return literalString(current);
}

function patternKey(pattern, name) {
  if (pattern?.type !== "ObjectPattern") return null;
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    let value = property.value;
    while (value?.type === "AssignmentPattern") value = value.left;
    if (value?.type === "Identifier" && value.name === name) {
      if (!property.computed) return staticName(property.key, false);
      const key = literalString(property.key);
      // undefined: this name's key is not static. null: the pattern does not bind it.
      return key == null ? undefined : key;
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

function isReactNamespace(node, context) {
  return namespaceStatus(node, context) === "yes";
}

function valueKind(node, context) {
  const variable = resolveVariable(node, context);
  const def = variable?.defs?.length === 1 ? variable.defs[0] : null;
  return def?.type === "Variable" ? def.parent?.kind : null;
}

function constStringKey(node, context) {
  const current = unwrapSequence(node);
  if (current?.type !== "Identifier" || valueKind(current, context) !== "const") return null;
  const declarator = resolveVariable(current, context)?.defs[0]?.node;
  if (declarator?.id?.type !== "Identifier" || !declarator.init) return null;
  return literalString(declarator.init);
}

function selectorName(property, computed, context) {
  return staticName(property, computed) || constStringKey(property, context);
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
    const name = selectorName(callee.property, callee.computed, context);
    if (name === "call" || name === "apply") callee = unwrapSequence(callee.object);
  }
  if (callee?.type === "Identifier") {
    const fact = importFact(callee, context);
    if (namedUse(fact)) {
      const kind = valueKind(callee, context);
      if (kind !== "let" && kind !== "var") return "yes";
      const variable = resolveVariable(callee, context);
      const written = variable?.references?.some(
        (reference) => reference.isWrite() && !reference.init,
      );
      return written ? "unresolved" : "yes";
    }
    if (fact?.unresolved) return "unresolved";
    if (fact) return "no";
    const variable = resolveVariable(callee, context);
    if (variable && seen.has(variable)) return "no";
    if (!variable || variable.defs.length === 0) {
      return useNames.has(callee.name) ? "yes" : "no";
    }
    if (variable.defs.length !== 1) return "no";
    const def = variable.defs[0];
    if (def?.type === "Variable" && def.node?.init) {
      const kind = def.node.parent?.kind;
      if (kind === "const") {
        seen.add(variable);
        if (def.node.id.type === "Identifier") {
          return reactUseStatus(def.node.init, context, useNames, seen);
        }
        const key = patternKey(def.node.id, callee.name);
        if (key === undefined)
          return namespaceStatus(def.node.init, context) === "no" ? "no" : "unresolved";
        if (key !== "use") return "no";
        return namespaceStatus(def.node.init, context);
      }
      if (def.node.id?.type === "Identifier") {
        seen.add(variable);
        const status = reactUseStatus(def.node.init, context, useNames, seen);
        if (status !== "no") return "unresolved";
      }
    }
    return "no";
  }
  if (callee?.type !== "MemberExpression") return "no";
  const name = selectorName(callee.property, callee.computed, context);
  if (name === "use") return namespaceStatus(callee.object, context);
  if (name) return "no";
  const status = namespaceStatus(callee.object, context);
  return status === "no" ? "no" : "unresolved";
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

function ordinaryUseCall(node, context) {
  const current = unwrapSequence(node);
  if (!current || current.optional || current.arguments.length !== 1) return false;
  if (current.arguments[0].type === "SpreadElement") return false;
  const callee = unwrapSequence(current.callee);
  return (
    callee?.type === "Identifier" ||
    (callee?.type === "MemberExpression" &&
      !callee.optional &&
      !["call", "apply"].includes(selectorName(callee.property, callee.computed, context)))
  );
}

function forwardedArgument(node, context) {
  const forwarded = unwrapSequence(node.callee);
  const name =
    forwarded?.type === "MemberExpression"
      ? selectorName(forwarded.property, forwarded.computed, context)
      : null;
  if (name !== "call" && name !== "apply") return node.arguments[0];
  if (name !== "apply") return node.arguments[1];
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
  constStringKey,
  patternKey,
  reactUseStatus,
  staticName,
};
