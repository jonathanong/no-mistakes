"use strict";

const { constStringKey } = require("./const-string-key");
const { literalString } = require("./logical-call");

function patternLocal(value) {
  while (value?.type === "AssignmentPattern") value = value.left;
  return value?.type === "Identifier" ? value : null;
}

function propertyKey(property, readKey, context) {
  if (!property.computed && property.key?.type === "Identifier") return property.key.name;
  return readKey(property.key) ?? constStringKey(property.key, context);
}

function patternPath(pattern, name, readKey, prefix = [], context) {
  if (pattern?.type === "AssignmentPattern")
    return patternPath(pattern.left, name, readKey, prefix, context);
  if (pattern?.type !== "ObjectPattern") return null;
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const key = propertyKey(property, readKey, context);
    const local = patternLocal(property.value);
    if (local?.name === name) return key === null ? undefined : [...prefix, key];
    if (property.value?.type !== "ObjectPattern" && property.value?.type !== "AssignmentPattern")
      continue;
    const found = patternPath(
      property.value,
      name,
      readKey,
      key === null ? prefix : [...prefix, key],
      context,
    );
    if (found === null) continue;
    if (key === null) return undefined;
    return found;
  }
  return null;
}

function factAtPath(source, names, namespaceMember) {
  let fact = source;
  for (const name of names) {
    if (!fact || fact.unresolved) return { unresolved: true };
    const member = namespaceMember(fact, name);
    if (member) {
      fact = fact.name && !member.name ? { ...member, name: `${fact.name}.${name}` } : member;
      continue;
    }
    if (fact.kind !== "named" || !(fact.name || fact.imported)) return null;
    fact = { ...fact, name: `${fact.name || fact.imported}.${name}`, imported: name };
  }
  return fact;
}

function memberChain(object, path) {
  return path.reduce(
    (current, name) => ({
      type: "MemberExpression",
      object: current,
      property: { type: "Identifier", name },
      computed: false,
      optional: false,
    }),
    object,
  );
}

function unresolvedDefault() {
  return { type: "ObjectExpression", properties: [] };
}

function collectDefaults(pattern, name, context, found) {
  if (!pattern) return;
  if (pattern.type === "AssignmentPattern") {
    const local = patternLocal(pattern);
    if (local?.name === name) found.push({ expression: pattern.right, path: [] });
    else {
      const path = patternPath(pattern.left, name, literalString, [], context);
      if (path === undefined) found.push({ unresolved: true });
      else if (path?.length) found.push({ expression: pattern.right, path });
    }
    collectDefaults(pattern.left, name, context, found);
    return;
  }
  if (pattern.type !== "ObjectPattern") return;
  for (const property of pattern.properties) {
    if (property.type === "Property") collectDefaults(property.value, name, context, found);
  }
}

function patternDefault(pattern, name, context) {
  const found = [];
  collectDefaults(pattern, name, context, found);
  if (found.length !== 1 || found[0].unresolved) return found.length ? unresolvedDefault() : null;
  const item = found[0];
  return item.path.length ? memberChain(item.expression, item.path) : item.expression;
}

function sameImportFact(left, right) {
  return Boolean(
    left &&
    right &&
    !left.unresolved &&
    !right.unresolved &&
    left.module === right.module &&
    left.kind === right.kind &&
    left.imported === right.imported &&
    (left.name == null || right.name == null || left.name === right.name) &&
    Boolean(left.typeOnly) === Boolean(right.typeOnly),
  );
}

function withPatternDefault(fact, alternate) {
  if (!alternate && !fact) return fact;
  if (sameImportFact(fact, alternate)) return fact;
  return alternate || fact ? { unresolved: true } : fact;
}

module.exports = { factAtPath, patternDefault, patternPath, withPatternDefault };
