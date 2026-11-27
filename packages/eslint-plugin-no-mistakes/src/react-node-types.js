"use strict";

const { literalString } = require("./rules/module-mock-helpers");
const { importFact } = require("./rules/import-resolution");

function syntacticRequire(init) {
  if (init?.type !== "CallExpression" || init.optional || init.arguments?.length !== 1) return null;
  if (init.callee?.type !== "Identifier" || init.callee.name !== "require") return null;
  const moduleName = literalString(init.arguments[0]);
  if (moduleName === null) return { unresolved: true };
  return { module: moduleName, kind: "namespace", typeOnly: false };
}
const {
  assertedType,
  bindPatternNames,
  definePattern,
  keyName,
  reactTypeStatus,
  typeAnnotation,
  typeName,
} = require("./react-node-type-query");

function createReactNodeFacts(program, context) {
  const reactNodeNames = new Set(["React.ReactNode"]);
  const unresolvedNamespaces = new Set();
  const unresolvedTypes = new Set();
  const aliases = new Map();
  const objectProps = new Map();
  const unresolvedObjectProps = new Map();
  const typeFacts = () => ({ aliases, reactNodeNames, unresolvedNamespaces, unresolvedTypes });

  function topLevelDeclaration(statement) {
    return (statement.type === "ExportNamedDeclaration" ||
      statement.type === "ExportDefaultDeclaration") &&
      statement.declaration
      ? statement.declaration
      : statement;
  }

  function setChanged(previous, next) {
    if (!previous || previous.size !== next.size) return true;
    for (const value of next) {
      if (!previous.has(value)) return true;
    }
    return false;
  }

  function heritageName(heritage) {
    const expression = heritage && heritage.expression;
    return expression && expression.type === "Identifier" ? expression.name : null;
  }

  function absorb(target, source) {
    if (!source) return;
    for (const value of source) target.add(value);
  }

  function collectMembers(members) {
    const props = new Set();
    const unresolved = new Set();
    for (const member of members || []) {
      if (member.type !== "TSPropertySignature") continue;
      const name = keyName(member.key);
      if (!name) continue;
      const status = reactTypeStatus(typeAnnotation(member), typeFacts());
      if (status === "yes") props.add(name);
      else if (status === "unresolved") unresolved.add(name);
    }
    return { props, unresolved };
  }

  for (const statement of program.body || []) {
    if (statement.type !== "ImportDeclaration" || statement.source.value !== "react") continue;
    for (const specifier of statement.specifiers || []) {
      if (specifier.type === "ImportSpecifier" && keyName(specifier.imported) === "ReactNode") {
        reactNodeNames.add(specifier.local.name);
      }
      if (
        specifier.type === "ImportNamespaceSpecifier" ||
        specifier.type === "ImportDefaultSpecifier" ||
        (specifier.type === "ImportSpecifier" && keyName(specifier.imported) === "default")
      ) {
        reactNodeNames.add(`${specifier.local.name}.ReactNode`);
      }
    }
  }

  for (const statement of program.body || []) {
    if (statement.type !== "TSImportEqualsDeclaration" || statement.id?.type !== "Identifier") {
      continue;
    }
    const reference = statement.moduleReference;
    const moduleName =
      reference?.type === "TSExternalModuleReference" ? literalString(reference.expression) : null;
    if (moduleName === "react") reactNodeNames.add(`${statement.id.name}.ReactNode`);
    else if (moduleName === null) unresolvedNamespaces.add(statement.id.name);
  }

  for (const statement of program.body || []) {
    const declaration = topLevelDeclaration(statement);
    if (declaration.type !== "VariableDeclaration" || declaration.kind !== "const") continue;
    for (const declarator of declaration.declarations || []) {
      if (declarator.id?.type !== "Identifier") continue;
      const fact = context?.sourceCode?.getScope
        ? importFact(declarator.init, context)
        : syntacticRequire(declarator.init);
      if (!fact || fact.typeOnly) continue;
      if (fact.unresolved) unresolvedNamespaces.add(declarator.id.name);
      else if (fact.module === "react" && (fact.kind === "namespace" || fact.kind === "default")) {
        reactNodeNames.add(`${declarator.id.name}.ReactNode`);
      }
    }
  }

  let changed = true;
  while (changed) {
    changed = false;
    for (const statement of program.body || []) {
      const declaration = topLevelDeclaration(statement);
      if (declaration.type !== "TSTypeAliasDeclaration") continue;
      const name = declaration.id.name;
      const status = reactTypeStatus(declaration.typeAnnotation, typeFacts());
      if (status === "yes" && !aliases.has(name)) {
        aliases.set(name, true);
        changed = true;
      } else if (status === "unresolved" && !unresolvedTypes.has(name)) {
        unresolvedTypes.add(name);
        changed = true;
      }
    }
  }

  let objectsChanged = true;
  while (objectsChanged) {
    objectsChanged = false;
    for (const statement of program.body || []) {
      const declaration = topLevelDeclaration(statement);
      const literal =
        declaration.type === "TSTypeAliasDeclaration" &&
        declaration.typeAnnotation.type === "TSTypeLiteral";
      if (declaration.type !== "TSInterfaceDeclaration" && !literal) continue;
      const collected = collectMembers(
        literal ? declaration.typeAnnotation.members : declaration.body && declaration.body.body,
      );
      const props = collected.props;
      const unresolved = collected.unresolved;
      absorb(props, objectProps.get(declaration.id.name));
      absorb(unresolved, unresolvedObjectProps.get(declaration.id.name));
      for (const heritage of declaration.extends || []) {
        const inheritedName = heritageName(heritage);
        absorb(props, objectProps.get(inheritedName));
        absorb(unresolved, unresolvedObjectProps.get(inheritedName));
      }
      if (setChanged(objectProps.get(declaration.id.name), props)) {
        objectProps.set(declaration.id.name, props);
        objectsChanged = true;
      }
      if (setChanged(unresolvedObjectProps.get(declaration.id.name), unresolved)) {
        unresolvedObjectProps.set(declaration.id.name, unresolved);
        objectsChanged = true;
      }
    }
  }

  return {
    aliases,
    objectProps,
    reactNodeNames,
    unresolvedNamespaces,
    unresolvedObjectProps,
    unresolvedTypes,
  };
}

module.exports = {
  assertedType,
  bindPatternNames,
  createReactNodeFacts,
  definePattern,
  keyName,
  reactTypeStatus,
  typeAnnotation,
  typeName,
};
