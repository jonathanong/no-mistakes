"use strict";

const { matchesConfiguredModule } = require("./postgres-scoped-executors");
const { declaredInConfiguredModule } = require("./postgres-no-executor-adapter-module");
const { importedExecutorOption } = require("./postgres-no-executor-adapter-reexport");
// TypeScript's SymbolFlags.Optional bit; type services supply the resolved symbol.
const OPTIONAL_SYMBOL_FLAG = 1 << 24;

function configuredImport(typeName, checker, options, seen, from) {
  const symbol = checker.getSymbolAtLocation(typeName);
  if (!symbol || seen.has(symbol) || symbol.declarations?.length !== 1) return false;
  seen.add(symbol);
  const declaration = symbol.declarations[0];
  const file = declaration.getSourceFile().fileName;
  if (
    declaration.name?.text &&
    declaration.parent?.statements &&
    options.executorTypeNames.includes(declaration.name.text) &&
    declaredInConfiguredModule(file, from, options)
  )
    return true;
  const source = declaration.parent?.parent?.parent?.moduleSpecifier?.text;
  if (typeof source === "string") {
    const name = declaration.propertyName?.text ?? declaration.name?.text;
    return (
      options.executorTypeNames.includes(name) &&
      matchesConfiguredModule(source, options.importSpecifier, declaration.getSourceFile().fileName)
    );
  }
  // Only an explicit type alias may carry the imported name through another hop.
  const next =
    declaration.parent?.statements && !declaration.parameters && declaration.type?.typeName;
  return Boolean(next && configuredImport(next, checker, options, seen, from));
}

// An optional parameter or an explicit `| undefined` / `| null` still declares
// one options object; any other union has no single contract to judge.
function optionObjectType(annotation, checker) {
  return checker.getNonNullableType(checker.getTypeFromTypeNode(annotation));
}

// A call site's inferred shape is insufficient: inspect the implementation's
// declared parameter, then the declaration of its named property.
function declaredOptionType(annotation, property, checker, options, supplied = false) {
  if (!annotation || !property) return false;
  const type = optionObjectType(annotation, checker);
  if (type.isUnion()) return false;
  const symbol = type.getProperty(property);
  if (symbol?.declarations?.length !== 1) return false;
  if (symbol.flags & OPTIONAL_SYMBOL_FLAG && !supplied) return false;
  const member = symbol.declarations[0];
  if (
    member.name?.text !== property ||
    (member.questionToken && !supplied) ||
    member.parameters ||
    !member.type?.typeName
  )
    return false;
  const from = annotation.getSourceFile().fileName;
  return (
    configuredImport(member.type.typeName, checker, options, new Set(), from) ||
    importedExecutorOption(annotation, property, type, checker, options)
  );
}

module.exports = { declaredOptionType, optionObjectType };
