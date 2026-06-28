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

function assertionType(node) {
  return node?.type === "TSAsExpression" || node?.type === "TSTypeAssertion"
    ? node.typeAnnotation
    : null;
}

function declarationOf(statement) {
  return (statement.type === "ExportNamedDeclaration" ||
    statement.type === "ExportDefaultDeclaration") &&
    statement.declaration
    ? statement.declaration
    : statement;
}

function heritageName(heritage) {
  const expression = heritage?.expression;
  return expression?.type === "Identifier" ? expression.name : null;
}

function collectNullableAliases(program) {
  const aliases = new Set();
  let changed = true;
  while (changed) {
    changed = false;
    for (const statement of program.body || []) {
      const declaration = declarationOf(statement);
      if (declaration.type !== "TSTypeAliasDeclaration") continue;
      if (
        !aliases.has(declaration.id.name) &&
        typeIncludesNull(declaration.typeAnnotation, aliases)
      ) {
        aliases.add(declaration.id.name);
        changed = true;
      }
    }
  }
  return aliases;
}

function collectTypeProps(program, options, patterns, typeProps) {
  const aliases = collectNullableAliases(program);
  const interfaces = new Map();
  const interfaceExtends = new Map();
  for (const statement of program.body || []) {
    const declaration = declarationOf(statement);
    if (
      declaration.type === "TSInterfaceDeclaration" &&
      optionTypeAllowed(declaration.id.name, options, patterns)
    ) {
      const props = interfaces.get(declaration.id.name) || new Set();
      for (const prop of nullablePropsFromMembers(declaration.body.body, aliases)) props.add(prop);
      interfaces.set(declaration.id.name, props);
      interfaceExtends.set(
        declaration.id.name,
        (interfaceExtends.get(declaration.id.name) || []).concat(
          (declaration.extends || []).map(heritageName).filter(Boolean),
        ),
      );
    }
    if (
      declaration.type === "TSTypeAliasDeclaration" &&
      optionTypeAllowed(declaration.id.name, options, patterns) &&
      declaration.typeAnnotation.type === "TSTypeLiteral"
    ) {
      typeProps.set(
        declaration.id.name,
        nullablePropsFromMembers(declaration.typeAnnotation.members, aliases),
      );
    }
  }
  function resolveInterface(name, seen = new Set()) {
    if (seen.has(name)) return interfaces.get(name) || new Set();
    seen.add(name);
    const props = new Set(interfaces.get(name) || []);
    for (const base of interfaceExtends.get(name) || []) {
      for (const prop of resolveInterface(base, seen)) props.add(prop);
    }
    return props;
  }
  for (const name of interfaces.keys()) typeProps.set(name, resolveInterface(name));
}

module.exports = {
  assertionType,
  collectTypeProps,
  compilePatterns,
  isIdentifier,
  memberRootAndProperty,
  nullablePropsFromMembers,
  objectPropertyName,
  optionTypeAllowed,
  propsFromType,
  typeIncludesNull,
};
