"use strict";

const { resolveVariable } = require("./postgres-query-text");
const { propertyText } = require("./postgres-no-executor-adapter-logic");
const { TS_ASSERTIONS } = require("./postgres-no-executor-adapter-binding");

function unwrapAssertions(node) {
  let current = node;
  while (current && TS_ASSERTIONS.has(current.type)) current = current.expression;
  return current;
}

function patternKey(property, context) {
  if (property.computed) return propertyText({ computed: true, property: property.key }, context);
  const key = property.key;
  if (key?.type === "Identifier") return key.name;
  if (key?.type === "Literal" && typeof key.value === "string") return key.value;
  return null;
}

function findBindingPath(pattern, target, context) {
  if (pattern === target) return [];
  if (pattern?.type !== "ObjectPattern") return null;
  const excluded = new Set();
  let hasDynamicSibling = false;
  for (const item of pattern.properties) {
    if (item.type !== "Property") continue;
    const key = patternKey(item, context);
    if (key) excluded.add(key);
    else hasDynamicSibling = true;
  }
  for (const item of pattern.properties) {
    if (item.type === "Property") {
      const key = patternKey(item, context);
      if (!key) continue;
      const tail = findBindingPath(item.value, target, context);
      if (tail) return [{ kind: "prop", key }, ...tail];
    } else if (!hasDynamicSibling) {
      const tail = findBindingPath(item.argument, target, context);
      if (tail) return [{ kind: "rest", excluded }, ...tail];
    }
  }
  return null;
}

function objectPropertyValue(objectExpr, key, context) {
  let result = null;
  for (const entry of objectExpr.properties) {
    if (entry.type === "SpreadElement") {
      result = { kind: "spread", node: entry.argument };
      continue;
    }
    const propKey = patternKey(entry, context);
    if (propKey === key) {
      result = entry.kind === "init" ? { kind: "value", node: entry.value } : null;
    } else if (propKey === null) {
      result = null;
    }
  }
  return result;
}

function constBinding(identifier, context, seen) {
  const variable = resolveVariable(identifier, context);
  if (!variable || variable.defs.length !== 1 || seen.has(variable)) return null;
  const def = variable.defs[0];
  if (def.type !== "Variable" || def.parent?.kind !== "const" || !def.node?.init) return null;
  const path = findBindingPath(def.node.id, def.name, context);
  return path ? { variable, init: def.node.init, path } : null;
}

function projectPath(expr, path, targetKey, context, seen) {
  const current = unwrapAssertions(expr);
  if (path.length === 0) return current;
  const [step, ...rest] = path;
  if (step.kind === "rest") {
    const nextProp = rest.find((item) => item.kind === "prop");
    const checkedKey = nextProp ? nextProp.key : targetKey;
    return step.excluded.has(checkedKey)
      ? null
      : projectPath(current, rest, targetKey, context, seen);
  }
  if (current?.type === "ObjectExpression") {
    const prop = objectPropertyValue(current, step.key, context);
    if (prop?.kind === "value") return projectPath(prop.node, rest, targetKey, context, seen);
    if (prop?.kind === "spread") return projectPath(prop.node, path, targetKey, context, seen);
    return null;
  }
  if (current?.type === "Identifier") {
    const binding = constBinding(current, context, seen);
    if (!binding) return null;
    const nextSeen = new Set(seen);
    nextSeen.add(binding.variable);
    return projectPath(binding.init, [...binding.path, ...path], targetKey, context, nextSeen);
  }
  return null;
}

function resolveNamespaceForKey(node, key, context, importSpecifier, isDirectNamespace, seen) {
  const current = unwrapAssertions(node);
  if (current?.type === "ObjectExpression") {
    const prop = objectPropertyValue(current, key, context);
    return (
      prop?.kind === "spread" &&
      resolveNamespaceForKey(prop.node, key, context, importSpecifier, isDirectNamespace, seen)
    );
  }
  if (current?.type !== "Identifier") return false;
  if (isDirectNamespace(current, context, importSpecifier)) return true;
  const binding = constBinding(current, context, seen);
  if (!binding) return false;
  const nextSeen = new Set(seen);
  nextSeen.add(binding.variable);
  const resolved = projectPath(binding.init, binding.path, key, context, nextSeen);
  return Boolean(
    resolved &&
    resolveNamespaceForKey(resolved, key, context, importSpecifier, isDirectNamespace, nextSeen),
  );
}

module.exports = {
  findBindingPath,
  projectPath,
  resolveNamespaceForKey,
};
