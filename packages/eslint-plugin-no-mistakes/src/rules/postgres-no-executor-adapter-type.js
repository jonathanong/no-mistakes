"use strict";

const { resolveVariable } = require("./postgres-query-text");
const { variableIsImportedExecutor } = require("./postgres-no-executor-adapter-binding");
const { matchesConfiguredModule } = require("./postgres-scoped-executors");

function importedTypeName(name, context, options) {
  const variable = resolveVariable(name, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  return Boolean(
    def?.type === "ImportBinding" &&
    def.node.type === "ImportSpecifier" &&
    options.executorTypeNames.includes(def.node.imported.name) &&
    matchesConfiguredModule(def.parent.source.value, options.importSpecifier, context.filename),
  );
}

function localExecutorType(type, context, options) {
  if (type?.type === "TSUnionType") {
    return (
      type.types.length > 0 && type.types.every((part) => localExecutorType(part, context, options))
    );
  }
  if (type?.type === "TSTypeReference" && type.typeName.type === "Identifier") {
    return importedTypeName(type.typeName, context, options);
  }
  if (type?.type === "TSTypeQuery" && type.exprName.type === "Identifier") {
    return variableIsImportedExecutor(resolveVariable(type.exprName, context), options);
  }
  return false;
}

function importedSpecifier(node, names, type, options, runtime = false) {
  const symbol = type.checker.getSymbolAtLocation(node);
  return Boolean(
    symbol?.declarations?.some((declaration) => {
      const source = declaration.parent?.parent?.parent?.moduleSpecifier?.text;
      const name = declaration.propertyName?.text ?? declaration.name?.text;
      return (
        typeof source === "string" &&
        (!runtime ||
          (!declaration.isTypeOnly &&
            !declaration.parent?.parent?.isTypeOnly &&
            (!options.importSpecifier || source === options.importSpecifier))) &&
        names.includes(name) &&
        matchesConfiguredModule(source, options.importSpecifier, type.sourceFile.fileName)
      );
    }),
  );
}

function importedExecutorType(node, checker, options) {
  const type = { checker, sourceFile: node.getSourceFile() };
  if (node.kind === require("typescript").SyntaxKind.UnionType) {
    return (
      node.types.length > 0 &&
      node.types.every((part) => importedExecutorType(part, checker, options))
    );
  }
  if (node.typeName) {
    return importedSpecifier(node.typeName, options.executorTypeNames, type, options);
  }
  if (node.exprName && node.getFirstToken()?.getText() === "typeof") {
    return importedSpecifier(node.exprName, options.executorNames, type, options, true);
  }
  return false;
}

module.exports = { localExecutorType, importedExecutorType };
