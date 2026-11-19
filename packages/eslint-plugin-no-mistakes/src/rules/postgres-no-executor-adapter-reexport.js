"use strict";

const { matchesConfiguredModule } = require("./postgres-scoped-executors");
// TypeScript's SymbolFlags.Alias bit; type services supply the resolved symbol.
const ALIAS_SYMBOL_FLAG = 1 << 21;

// The top-level declaration a type reference ends at, followed through
// import and re-export aliases to the declaring package.
function declarationOf(typeName, checker) {
  let symbol = checker.getSymbolAtLocation(typeName);
  if (symbol.flags & ALIAS_SYMBOL_FLAG) symbol = checker.getAliasedSymbol(symbol);
  return symbol?.declarations?.[0];
}

function namesExecutor(member, checker, options) {
  const typeName = member.type?.typeName;
  const declaration = typeName && declarationOf(typeName, checker);
  return Boolean(
    declaration?.parent?.statements && options.executorTypeNames.includes(declaration.name?.text),
  );
}

// An options type imported from the configured module whose `property` is
// declared with a named executor type, wherever that type is declared; see
// "Library executor terminals" in docs/eslint-rules/postgres-no-executor-adapter.md.
// Any other property typed as an executor keeps the contract ambiguous.
function importedExecutorOption(annotation, property, type, checker, options) {
  const symbol = annotation.typeName && checker.getSymbolAtLocation(annotation.typeName);
  const source = symbol?.declarations?.[0]?.parent?.parent?.parent?.moduleSpecifier?.text;
  if (
    typeof source !== "string" ||
    !matchesConfiguredModule(source, options.importSpecifier, annotation.getSourceFile().fileName)
  )
    return false;
  return checker.getPropertiesOfType(type).every((other) => {
    const executor = namesExecutor(other.declarations[0], checker, options);
    return other.name === property ? executor : !executor;
  });
}

module.exports = { importedExecutorOption };
