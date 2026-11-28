"use strict";

function patternLocal(value) {
  while (value?.type === "AssignmentPattern") value = value.left;
  return value?.type === "Identifier" ? value : null;
}

function propertyKey(property, literalString) {
  if (!property.computed && property.key?.type === "Identifier") return property.key.name;
  return literalString(property.key);
}

function patternPath(pattern, name, literalString, prefix = []) {
  if (pattern?.type === "AssignmentPattern")
    return patternPath(pattern.left, name, literalString, prefix);
  if (pattern?.type !== "ObjectPattern") return null;
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const key = propertyKey(property, literalString);
    const local = patternLocal(property.value);
    if (local?.name === name) return key === null ? undefined : [...prefix, key];
    if (property.value?.type !== "ObjectPattern" && property.value?.type !== "AssignmentPattern")
      continue;
    const found = patternPath(
      property.value,
      name,
      literalString,
      key === null ? prefix : [...prefix, key],
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

function patternDefault(pattern, name) {
  if (pattern?.type === "AssignmentPattern") return patternDefault(pattern.left, name);
  if (pattern?.type !== "ObjectPattern") return null;
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const value = property.value;
    if (value?.type === "AssignmentPattern" && patternLocal(value)?.name === name)
      return value.right ?? null;
    if (value?.type !== "ObjectPattern" && value?.type !== "AssignmentPattern") continue;
    const found = patternDefault(value, name);
    if (found) return found;
  }
  return null;
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
    Boolean(left.typeOnly) === Boolean(right.typeOnly),
  );
}

function withPatternDefault(fact, alternate) {
  if (!alternate && !fact) return fact;
  if (sameImportFact(fact, alternate)) return fact;
  return alternate || fact ? { unresolved: true } : fact;
}

module.exports = { factAtPath, patternDefault, patternPath, withPatternDefault };
