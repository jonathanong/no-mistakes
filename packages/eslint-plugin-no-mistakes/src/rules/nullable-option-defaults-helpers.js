"use strict";

const { keyName, typeAnnotation, typeName } = require("../react-node-types");
const { memberPropertyName } = require("./module-mock-helpers");
function compilePatterns(patterns = []) {
  return patterns.flatMap((pattern) => {
    try {
      return [new RegExp(pattern)];
    } catch {
      return [];
    }
  });
}
function unwrapType(node) {
  let current = node;
  while (
    current &&
    (current.type === "TSParenthesizedType" ||
      current.type === "TSOptionalType" ||
      current.type === "TSRestType")
  ) {
    current = current.typeAnnotation;
  }
  return current;
}
function typeIncludesNull(node, aliases = new Set()) {
  const current = unwrapType(node);
  if (!current) return false;
  if (current.type === "TSNullKeyword") return true;
  const name = typeName(current);
  if (name && aliases.has(name)) return true;
  if (current.type === "TSUnionType") {
    return current.types.some((item) => typeIncludesNull(item, aliases));
  }
  return false;
}
function optionTypeAllowed(name, options, patterns) {
  if (!name) return true;
  const names = options.optionObjectNames ?? [];
  const rawPatterns = options.optionObjectNamePatterns ?? [];
  if (names.length === 0 && rawPatterns.length === 0) return true;
  return names.includes(name) || patterns.some((pattern) => pattern.test(name));
}
function nullablePropsFromMembers(members, aliases = new Set()) {
  const props = new Set();
  for (const member of members || []) {
    if (member.type !== "TSPropertySignature" || member.optional !== true) continue;
    if (!typeIncludesNull(typeAnnotation(member), aliases)) continue;
    const name = keyName(member.key);
    if (name) props.add(name);
  }
  return props;
}
function propNamesFromMembers(members) {
  const props = new Set();
  for (const member of members || []) {
    if (member.type !== "TSPropertySignature") continue;
    const name = keyName(member.key);
    if (name) props.add(name);
  }
  return props;
}
function propsFromType(type, facts) {
  const current = unwrapType(type);
  if (!current) return null;
  if (current.type === "TSTypeLiteral") {
    return nullablePropsFromMembers(current.members, facts.aliases);
  }
  if (typeName(current) === "Readonly") {
    return propsFromType(typeArguments(current)[0], facts);
  }
  if (current.type === "TSUnionType" || current.type === "TSIntersectionType") {
    return propsFromTypes(current.types, facts);
  }
  const name = typeName(current);
  return name ? resolveTypeProps(name, facts) : null;
}
function typeArguments(node) {
  return node.typeArguments?.params || node.typeParameters?.params || [];
}
function propsFromTypes(types, facts) {
  const props = new Set();
  for (const item of types) {
    const itemProps = propsFromType(item, facts);
    for (const prop of itemProps || []) props.add(prop);
  }
  return props.size > 0 ? props : null;
}
const isIdentifier = (node) => node?.type === "Identifier";
const objectPropertyName = (property) =>
  property.type === "Property" ? keyName(property.key) : null;
function memberRootAndProperty(node) {
  let current = node;
  while (
    current?.type === "ChainExpression" ||
    current?.type === "TSAsExpression" ||
    current?.type === "TSTypeAssertion" ||
    current?.type === "TSNonNullExpression"
  ) {
    current = current.expression;
  }
  if (current?.type !== "MemberExpression") return null;
  const property = memberPropertyName(current);
  let object = current.object;
  while (object?.type === "ChainExpression" || object?.type === "TSNonNullExpression") {
    object = object.expression;
  }
  return isIdentifier(object) && property ? { object: object.name, property } : null;
}
function assertionType(node) {
  return node?.type === "TSAsExpression" || node?.type === "TSTypeAssertion"
    ? node.typeAnnotation
    : null;
}
function resolveTypeProps(name, facts, seen = new Set()) {
  if (seen.has(name)) return null;
  seen.add(name);
  const props = facts.typeProps.get(name);
  if (props) return props;
  const target = facts.objectAliases.get(name);
  if (target) return resolveTypeProps(target, facts, seen);
  return facts.includeAll === true ? facts.allTypeProps?.get(name) || null : null;
}

module.exports = {
  assertionType,
  compilePatterns,
  isIdentifier,
  memberRootAndProperty,
  nullablePropsFromMembers,
  objectPropertyName,
  optionTypeAllowed,
  propNamesFromMembers,
  propsFromType,
  typeIncludesNull,
};
