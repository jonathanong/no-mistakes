"use strict";

const { unwrapSequence } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { constStringKey } = require("./const-string-key");
const { classifiedProps } = require("./react-nullish-props");
const {
  assertedType,
  computedKey,
  keyName,
  reactTypeStatus,
  typeAnnotation,
} = require("../react-node-types");

function patternPropertyKey(pattern, name, context) {
  if (pattern?.type !== "ObjectPattern") return null;
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const value =
      property.value?.type === "AssignmentPattern" ? property.value.left : property.value;
    if (value?.type !== "Identifier" || value.name !== name) continue;
    if (!property.computed) return keyName(property.key);
    const key = computedKey(property.key) ?? constStringKey(property.key, context);
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

function recordedKeyStatus(init, key, lookup) {
  const current = unwrapSequence(init);
  if (current?.type !== "Identifier" || !lookup) return null;
  const props = lookup(current, "objectTypes");
  const unresolved = lookup(current, "unresolvedMembers");
  if (key === undefined)
    return props || unresolved ? (props?.size || unresolved?.size ? "unresolved" : "no") : null;
  if (props?.has(key)) return "yes";
  if (unresolved?.has(key)) return "unresolved";
  return props || unresolved ? "no" : null;
}

function forwardPatternStatus(pattern, name, init, context, facts, annotation, lookup) {
  const key = patternPropertyKey(pattern, name, context);
  if (key === null) return "no";
  const type = annotation || initObjectType(init, context);
  const classified = type ? classifiedProps(type, facts) : null;
  if (classified?.props || classified?.unresolved) {
    if (key === undefined)
      return classified.props?.size || classified.unresolved?.size ? "unresolved" : "no";
    if (classified.props?.has(key)) return "yes";
    if (classified.unresolved?.has(key)) return "unresolved";
    return "no";
  }
  if (type && reactTypeStatus(type, facts) === "unresolved") return "unresolved";
  return recordedKeyStatus(init, key, lookup) ?? (key === undefined ? "unresolved" : "no");
}

function hasOwnAnnotation(def) {
  if (def.type !== "Variable") return false;
  if (def.node?.id?.type === "ObjectPattern") return Boolean(typeAnnotation(def.node.id));
  return def.node?.id?.type === "Identifier" && Boolean(typeAnnotation(def.node.id));
}

function laterBindingStatus(
  identifier,
  context,
  reactTypeStatus,
  expressionStatus,
  facts,
  seen,
  lookup,
) {
  const variable = resolveVariable(identifier, context);
  const defs = (variable?.defs ?? []).filter(
    (def) => def.type !== "Type" && def.type !== "TypeParameter",
  );
  const laterDefs = defs.filter((def) => def.node?.range?.[0] > identifier.range[0]);
  const forwardValue = laterDefs.some((def) => def.type === "Variable");
  const active = forwardValue ? laterDefs : defs;
  const annotationDefs =
    forwardValue && !laterDefs.some(hasOwnAnnotation)
      ? defs.filter(
          (def) =>
            def.type === "Variable" &&
            (hasOwnAnnotation(def) || def.node?.range?.[0] > identifier.range[0]),
        )
      : active;
  if (!active.length || seen.has(variable))
    return seen.has(variable) && active.length ? "unresolved" : null;
  if (!forwardValue && active.some((def) => !(def.node?.range?.[0] > identifier.range[0])))
    return null;
  seen.add(variable);
  try {
    let annotated = false;
    let unresolved = false;
    for (const def of annotationDefs) {
      if (def.type !== "Variable") continue;
      if (def.node?.id?.type === "ObjectPattern") {
        const patternType = typeAnnotation(def.node.id);
        if (!patternType) continue;
        annotated = true;
        const status = forwardPatternStatus(
          def.node.id,
          identifier.name,
          null,
          context,
          facts,
          patternType,
          lookup,
        );
        if (status === "yes") return "yes";
        if (status === "unresolved") unresolved = true;
        continue;
      }
      if (def.node?.id?.type !== "Identifier") continue;
      const type = typeAnnotation(def.node.id);
      if (!type) continue;
      annotated = true;
      const status = reactTypeStatus(type, facts);
      if (status === "yes") return "yes";
      if (status === "unresolved") unresolved = true;
    }
    if (annotated) return unresolved ? "unresolved" : "no";
    const written = (variable.references ?? []).some(
      (reference) => reference.isWrite() && !reference.init,
    );
    for (const def of active) {
      if (def.type !== "Variable") continue;
      if (def.node?.id?.type === "ObjectPattern") {
        const status = forwardPatternStatus(
          def.node.id,
          identifier.name,
          def.node.init,
          context,
          facts,
          null,
          lookup,
        );
        if (status === "yes") return written ? "unresolved" : "yes";
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
      if (status === "yes") return written ? "unresolved" : "yes";
      if (status === "unresolved") unresolved = true;
    }
    return unresolved ? "unresolved" : "no";
  } finally {
    seen.delete(variable);
  }
}

module.exports = { laterBindingStatus };
