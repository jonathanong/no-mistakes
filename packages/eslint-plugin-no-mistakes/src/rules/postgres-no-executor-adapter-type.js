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

function namespaceMatches(source, name, typeOnly, options, filename, runtime) {
  const names = runtime ? options.executorNames : options.executorTypeNames;
  return Boolean(
    typeof source === "string" &&
    names.includes(name) &&
    matchesConfiguredModule(source, options.importSpecifier, filename) &&
    (!runtime || (!typeOnly && (!options.importSpecifier || source === options.importSpecifier))),
  );
}

function localNamespace(name, context, options, runtime) {
  if (name?.type !== "TSQualifiedName" || name.left.type !== "Identifier") return false;
  const variable = resolveVariable(name.left, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  return Boolean(
    def?.type === "ImportBinding" &&
    def.node.type === "ImportNamespaceSpecifier" &&
    namespaceMatches(
      def.parent.source.value,
      name.right.name,
      def.parent.importKind === "type",
      options,
      context.filename,
      runtime,
    ),
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
  if (type?.type === "TSTypeReference") {
    return localNamespace(type.typeName, context, options, false);
  }
  if (type?.type === "TSTypeQuery") {
    return localNamespace(type.exprName, context, options, true);
  }
  return false;
}

function importedNamespace(node, type, options, runtime) {
  const ts = require("typescript");
  if (!ts.isIdentifier(node.left)) return false;
  const symbol = type.checker.getSymbolAtLocation(node.left);
  return Boolean(
    symbol?.declarations?.some(
      (declaration) =>
        ts.isNamespaceImport(declaration) &&
        namespaceMatches(
          declaration.parent.parent.moduleSpecifier.text,
          node.right.text,
          declaration.parent.isTypeOnly,
          options,
          type.sourceFile.fileName,
          runtime,
        ),
    ),
  );
}

function importedSpecifier(node, names, type, options, runtime = false) {
  if (require("typescript").isQualifiedName(node)) {
    return importedNamespace(node, type, options, runtime);
  }
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
  const ts = require("typescript");
  if (ts.isParenthesizedTypeNode(node)) {
    return importedExecutorType(node.type, checker, options);
  }
  if (ts.isUnionTypeNode(node)) {
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

module.exports = { localExecutorType, importedExecutorType, importedSpecifier };
