"use strict";

const { literalString } = require("./rules/module-mock-helpers");
const { unwrapExpression } = require("./rules/async-ast");
const {
  definePattern,
  keyName,
  reactTypeStatus,
  typeAnnotation,
  typeName,
} = require("./react-node-type-query");

function createReactNodeFacts(program) {
  const reactNodeNames = new Set(["React.ReactNode"]);
  const unresolvedNamespaces = new Set();
  const unresolvedTypes = new Set();
  const aliases = new Map();
  const objectProps = new Map();
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

  function isReactNodeType(type) {
    return reactTypeStatus(type, typeFacts()) === "yes";
  }

  function collectMembers(members) {
    const props = new Set();
    for (const member of members || []) {
      if (member.type !== "TSPropertySignature" || !isReactNodeType(typeAnnotation(member))) {
        continue;
      }
      const name = keyName(member.key);
      if (name) props.add(name);
    }
    return props;
  }

  for (const statement of program.body || []) {
    if (statement.type !== "ImportDeclaration" || statement.source.value !== "react") continue;
    for (const specifier of statement.specifiers || []) {
      if (specifier.type === "ImportSpecifier" && keyName(specifier.imported) === "ReactNode") {
        reactNodeNames.add(specifier.local.name);
      }
      if (
        specifier.type === "ImportNamespaceSpecifier" ||
        specifier.type === "ImportDefaultSpecifier"
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
      const call = unwrapExpression(declarator.init);
      if (
        call?.type !== "CallExpression" ||
        call.optional ||
        unwrapExpression(call.callee)?.type !== "Identifier" ||
        unwrapExpression(call.callee)?.name !== "require" ||
        call.arguments.length !== 1
      ) {
        continue;
      }
      const moduleName = literalString(call.arguments[0]);
      if (moduleName === "react") reactNodeNames.add(`${declarator.id.name}.ReactNode`);
      else if (moduleName === null) unresolvedNamespaces.add(declarator.id.name);
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
      if (declaration.type === "TSInterfaceDeclaration") {
        const props = collectMembers(declaration.body && declaration.body.body);
        for (const heritage of declaration.extends || []) {
          const inherited = objectProps.get(heritageName(heritage));
          if (inherited) {
            for (const prop of inherited) props.add(prop);
          }
        }
        if (setChanged(objectProps.get(declaration.id.name), props)) {
          objectProps.set(declaration.id.name, props);
          objectsChanged = true;
        }
      }
      if (
        declaration.type === "TSTypeAliasDeclaration" &&
        declaration.typeAnnotation.type === "TSTypeLiteral"
      ) {
        const props = collectMembers(declaration.typeAnnotation.members);
        if (setChanged(objectProps.get(declaration.id.name), props)) {
          objectProps.set(declaration.id.name, props);
          objectsChanged = true;
        }
      }
    }
  }

  return { aliases, objectProps, reactNodeNames, unresolvedNamespaces, unresolvedTypes };
}

module.exports = {
  createReactNodeFacts,
  definePattern,
  keyName,
  reactTypeStatus,
  typeAnnotation,
  typeName,
};
