"use strict";

const { keyName, typeAnnotation, typeName } = require("../react-node-types");
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
const VALUE_WRAPPERS = new Set([
  "ChainExpression",
  "TSAsExpression",
  "TSTypeAssertion",
  "TSNonNullExpression",
  "TSSatisfiesExpression",
  "TSInstantiationExpression",
]);
function unwrapValue(node) {
  let current = node;
  while (VALUE_WRAPPERS.has(current?.type)) current = current.expression;
  return current;
}
function staticKey(node) {
  const current = unwrapValue(node);
  if (!current) return null;
  if (typeof current.name === "string" && current.type !== "Literal" && current.type !== "TemplateLiteral") {
    return current.name;
  }
  if (current.type === "Literal") return String(current.value);
  if (current.type === "TemplateLiteral" && current.expressions.length === 0) {
    return current.quasis.map((quasi) => quasi.value.cooked ?? quasi.value.raw).join("");
  }
  return null;
}
const objectPropertyName = (property) => {
  if (property.type !== "Property") return null;
  if (property.computed && property.key?.type === "Identifier") return null;
  return staticKey(property.key);
};
function memberRootAndProperty(node) {
  const current = unwrapValue(node);
  if (current?.type !== "MemberExpression") return null;
  const propertyNode = unwrapValue(current.property);
  if (current.computed && propertyNode?.type === "Identifier") return null;
  const property = staticKey(propertyNode);
  const object = unwrapValue(current.object);
  return isIdentifier(object) && property ? { object: object.name, property } : null;
}
function assertionType(node) {
  let current = node;
  while (current) {
    if (
      current.type === "TSAsExpression" ||
      current.type === "TSTypeAssertion" ||
      current.type === "TSSatisfiesExpression"
    ) {
      return current.typeAnnotation;
    }
    if (!VALUE_WRAPPERS.has(current.type)) return null;
    current = current.expression;
  }
  return null;
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
  unwrapValue,
  nullablePropsFromMembers,
  objectPropertyName,
  optionTypeAllowed,
  propNamesFromMembers,
  propsFromType,
  typeIncludesNull,
};
