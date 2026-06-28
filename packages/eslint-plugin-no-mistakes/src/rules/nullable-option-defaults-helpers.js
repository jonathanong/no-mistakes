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

function typeIncludesNull(node) {
  const current = unwrapType(node);
  if (!current) return false;
  if (current.type === "TSNullKeyword") return true;
  if (current.type === "TSUnionType") {
    return current.types.some((item) => typeIncludesNull(item));
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

function nullablePropsFromMembers(members) {
  const props = new Set();
  for (const member of members || []) {
    if (member.type !== "TSPropertySignature" || member.optional !== true) continue;
    if (!typeIncludesNull(typeAnnotation(member))) continue;
    const name = keyName(member.key);
    if (name) props.add(name);
  }
  return props;
}

function propsFromType(type, facts) {
  const current = unwrapType(type);
  if (!current) return null;
  if (current.type === "TSTypeLiteral") return nullablePropsFromMembers(current.members);
  const name = typeName(current);
  return name ? facts.typeProps.get(name) : null;
}

function isIdentifier(node) {
  return node?.type === "Identifier";
}

function objectPropertyName(property) {
  return property.type === "Property" ? keyName(property.key) : null;
}

function memberRootAndProperty(node) {
  let current = node;
  if (current?.type === "ChainExpression") current = current.expression;
  if (current?.type !== "MemberExpression") return null;
  const property = memberPropertyName(current);
  let object = current.object;
  if (object?.type === "ChainExpression") object = object.expression;
  return isIdentifier(object) && property ? { object: object.name, property } : null;
}

module.exports = {
  compilePatterns,
  isIdentifier,
  memberRootAndProperty,
  nullablePropsFromMembers,
  objectPropertyName,
  optionTypeAllowed,
  propsFromType,
  typeIncludesNull,
};
