"use strict";

const { unwrapSequence } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { constStringKey } = require("./const-string-key");
const { classifiedProps } = require("./react-nullish-props");
const { assertedType, computedKey, keyName, typeAnnotation } = require("../react-node-types");

function patternPropertyKey(pattern, name, context) {
  if (pattern?.type !== "ObjectPattern") return null;
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const value =
      property.value?.type === "AssignmentPattern" ? property.value.left : property.value;
    if (value?.type !== "Identifier" || value.name !== name) continue;
    if (!property.computed) return keyName(property.key);
    const key = computedKey(property.key) || constStringKey(property.key, context);
    return key == null ? undefined : key;
  }
  return null;
}

function annotatedType(node) {
  return typeAnnotation(node) || typeAnnotation(node?.left) || typeAnnotation(node?.id) || null;
}

function initObjectType(init, context) {
  const asserted = assertedType(init);
  if (asserted) return asserted;
  const current = unwrapSequence(init);
  const direct = typeAnnotation(current);
  if (direct) return direct;
  if (current?.type !== "Identifier") return null;
  const variable = resolveVariable(current, context);
  const def = variable?.defs?.length === 1 ? variable.defs[0] : null;
  return def ? annotatedType(def.name) || annotatedType(def.node) : null;
}

function forwardPatternStatus(pattern, name, init, context, facts) {
  const key = patternPropertyKey(pattern, name, context);
  if (key === null) return "no";
  const type = initObjectType(init, context);
  const classified = type ? classifiedProps(type, facts) : null;
  if (key === undefined) {
    if (!classified) return "unresolved";
    return classified.props?.size || classified.unresolved?.size ? "unresolved" : "no";
  }
  if (classified?.props?.has(key)) return "yes";
  if (classified?.unresolved?.has(key)) return "unresolved";
  return "no";
}

function laterBindingStatus(identifier, context, reactTypeStatus, expressionStatus, facts, seen) {
  const variable = resolveVariable(identifier, context);
  const defs = variable?.defs ?? [];
  if (!defs.length || seen.has(variable)) return seen.has(variable) ? "unresolved" : null;
  if (!defs.every((def) => def.node?.range?.[0] > identifier.range[0])) return null;
  seen.add(variable);
  try {
    if ((variable.references ?? []).some((reference) => reference.isWrite() && !reference.init)) {
      return "unresolved";
    }
    let unresolved = false;
    for (const def of defs) {
      if (def.type !== "Variable") continue;
      if (def.node?.id?.type === "ObjectPattern") {
        const status = forwardPatternStatus(
          def.node.id,
          identifier.name,
          def.node.init,
          context,
          facts,
        );
        if (status === "yes") return "yes";
        if (status === "unresolved") unresolved = true;
        continue;
      }
      if (def.node?.id?.type !== "Identifier") continue;
      const type = typeAnnotation(def.node.id);
      const status = type
        ? reactTypeStatus(type, facts)
        : def.node.init
          ? expressionStatus(def.node.init, seen)
          : "no";
      if (status === "yes") return "yes";
      if (status === "unresolved") unresolved = true;
    }
    return unresolved ? "unresolved" : "no";
  } finally {
    seen.delete(variable);
  }
}

module.exports = { laterBindingStatus };
