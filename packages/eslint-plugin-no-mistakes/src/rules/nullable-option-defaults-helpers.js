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

function reportDefaultsInPattern(context, pattern, props) {
  if (!pattern || pattern.type !== "ObjectPattern" || !props) return;
  for (const property of pattern.properties || []) {
    const name = objectPropertyName(property);
    if (!name || !props.has(name)) continue;
    if (property.value?.type === "AssignmentPattern") {
      context.report({ node: property.value, messageId: "default", data: { name } });
    }
  }
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

function createScope(kind) {
  return {
    bindings: new Set(),
    kind,
    nullableBindings: new Set(),
    objectProps: new Map(),
  };
}

function variableScope(scopes, currentScope, node) {
  if (!node.parent || node.parent.kind !== "var") return currentScope();
  for (let index = scopes.length - 1; index >= 0; index -= 1) {
    if (scopes[index].kind === "function" || scopes[index].kind === "program") {
      return scopes[index];
    }
  }
  return currentScope();
}

function objectProps(scopes, name) {
  for (let index = scopes.length - 1; index >= 0; index -= 1) {
    if (scopes[index].bindings.has(name) && !scopes[index].objectProps.has(name)) return null;
    const props = scopes[index].objectProps.get(name);
    if (props) return props;
  }
  return null;
}

function isNullableBinding(scopes, name) {
  for (let index = scopes.length - 1; index >= 0; index -= 1) {
    if (scopes[index].bindings.has(name)) {
      return scopes[index].nullableBindings.has(name);
    }
  }
  return false;
}

function assertionType(node) {
  return node?.type === "TSAsExpression" || node?.type === "TSTypeAssertion"
    ? node.typeAnnotation
    : null;
}

function collectTypeProps(program, options, patterns, typeProps) {
  for (const statement of program.body || []) {
    const declaration =
      (statement.type === "ExportNamedDeclaration" ||
        statement.type === "ExportDefaultDeclaration") &&
      statement.declaration
        ? statement.declaration
        : statement;
    if (
      declaration.type === "TSInterfaceDeclaration" &&
      optionTypeAllowed(declaration.id.name, options, patterns)
    ) {
      const props = typeProps.get(declaration.id.name) || new Set();
      for (const prop of nullablePropsFromMembers(declaration.body.body)) props.add(prop);
      typeProps.set(declaration.id.name, props);
    }
    if (
      declaration.type === "TSTypeAliasDeclaration" &&
      optionTypeAllowed(declaration.id.name, options, patterns) &&
      declaration.typeAnnotation.type === "TSTypeLiteral"
    ) {
      typeProps.set(
        declaration.id.name,
        nullablePropsFromMembers(declaration.typeAnnotation.members),
      );
    }
  }
}

module.exports = {
  assertionType,
  collectTypeProps,
  compilePatterns,
  createScope,
  isIdentifier,
  isNullableBinding,
  memberRootAndProperty,
  nullablePropsFromMembers,
  objectProps,
  objectPropertyName,
  optionTypeAllowed,
  propsFromType,
  reportDefaultsInPattern,
  typeIncludesNull,
  variableScope,
};
