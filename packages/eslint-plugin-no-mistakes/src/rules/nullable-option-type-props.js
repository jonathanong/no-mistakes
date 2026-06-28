"use strict";

const { typeName } = require("../react-node-types");
const {
  nullablePropsFromMembers,
  optionTypeAllowed,
  propNamesFromMembers,
  propsFromType,
  typeIncludesNull,
} = require("./nullable-option-defaults-helpers");

function declarationOf(statement) {
  return (statement.type === "ExportNamedDeclaration" ||
    statement.type === "ExportDefaultDeclaration") &&
    statement.declaration
    ? statement.declaration
    : statement;
}
const heritageName = (heritage) =>
  heritage?.expression?.type === "Identifier" ? heritage.expression.name : null;

function createTypeFacts() {
  return {
    aliases: new Set(),
    allTypeProps: new Map(),
    objectAliases: new Map(),
    typeProps: new Map(),
  };
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

function collectTypeProps(program, options, patterns, facts) {
  const aliases = collectNullableAliases(program);
  facts.aliases = aliases;
  const interfaceDeclared = new Map();
  const interfaces = new Map();
  const interfaceExtends = new Map();
  const typeAliases = new Map();
  for (const statement of program.body || []) {
    const declaration = declarationOf(statement);
    if (declaration.type === "TSInterfaceDeclaration") {
      const props = interfaces.get(declaration.id.name) || new Set();
      for (const prop of nullablePropsFromMembers(declaration.body.body, aliases)) props.add(prop);
      interfaceDeclared.set(declaration.id.name, propNamesFromMembers(declaration.body.body));
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
      declaration.typeAnnotation.type === "TSTypeLiteral"
    ) {
      const props = nullablePropsFromMembers(declaration.typeAnnotation.members, aliases);
      facts.allTypeProps.set(declaration.id.name, props);
      if (optionTypeAllowed(declaration.id.name, options, patterns)) {
        facts.typeProps.set(declaration.id.name, props);
      }
    } else if (declaration.type === "TSTypeAliasDeclaration") {
      typeAliases.set(declaration.id.name, declaration.typeAnnotation);
      const target = typeName(declaration.typeAnnotation);
      if (target) facts.objectAliases.set(declaration.id.name, target);
    }
  }
  function resolveInterface(name, seen = new Set()) {
    if (seen.has(name)) return interfaces.get(name) || new Set();
    seen.add(name);
    const props = new Set();
    for (const base of interfaceExtends.get(name) || []) {
      for (const prop of resolveInterface(base, seen)) props.add(prop);
    }
    for (const prop of interfaceDeclared.get(name) || []) props.delete(prop);
    for (const prop of interfaces.get(name) || []) props.add(prop);
    return props;
  }
  for (const name of interfaces.keys()) {
    const props = resolveInterface(name);
    facts.allTypeProps.set(name, props);
    if (optionTypeAllowed(name, options, patterns)) facts.typeProps.set(name, props);
  }
  for (const [name, type] of typeAliases) {
    if (!optionTypeAllowed(name, options, patterns) || facts.typeProps.has(name)) continue;
    const props = propsFromType(type, { ...facts, includeAll: true });
    if (props) facts.typeProps.set(name, props);
  }
}

module.exports = { collectTypeProps, createTypeFacts };
