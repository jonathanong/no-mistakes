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
const { assignReactObjectFacts } = require("./react-node-object-facts");
const {
  assertedType,
  bindPatternNames,
  computedKey,
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
    const declaration = topLevelDeclaration(statement);
    if (declaration.type !== "TSImportEqualsDeclaration" || declaration.id?.type !== "Identifier") {
      continue;
    }
    const reference = declaration.moduleReference;
    const moduleName =
      reference?.type === "TSExternalModuleReference" ? literalString(reference.expression) : null;
    if (moduleName === "react") reactNodeNames.add(`${declaration.id.name}.ReactNode`);
    else if (moduleName === null) unresolvedNamespaces.add(declaration.id.name);
  }

  for (const statement of program.body || []) {
    const declaration = topLevelDeclaration(statement);
    if (declaration.type !== "VariableDeclaration") {
      continue;
    }
    for (const declarator of declaration.declarations || []) {
      if (declarator.id?.type !== "Identifier") continue;
      const fact = context?.sourceCode?.getScope
        ? importFact(declarator.id, context)
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

  assignReactObjectFacts(
    program,
    topLevelDeclaration,
    typeFacts(),
    objectProps,
    unresolvedObjectProps,
  );

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
  computedKey,
  createReactNodeFacts,
  definePattern,
  keyName,
  reactTypeStatus,
  typeAnnotation,
  typeName,
};
