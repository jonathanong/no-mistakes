"use strict";

const { matchesConfiguredModule } = require("./postgres-scoped-executors");
// TypeScript's SymbolFlags.Optional bit; type services supply the resolved symbol.
const OPTIONAL_SYMBOL_FLAG = 1 << 24;

// A call site's inferred shape is insufficient: inspect the implementation's
// declared parameter, then the declaration of its named property.
function declaredOptionType(annotation, property, checker, options) {
  if (!annotation || !property) return false;
  const type = checker.getTypeFromTypeNode(annotation);
  if (type.isUnionOrIntersection()) return false;
  const symbol = type.getProperty(property);
  if (symbol?.declarations?.length !== 1) return false;
  if (symbol.flags & OPTIONAL_SYMBOL_FLAG) return false;
  const member = symbol.declarations[0];
  if (
    member.name?.text !== property ||
    member.questionToken ||
    member.parameters ||
    !member.type?.typeName
  )
    return false;
  const imported = checker.getSymbolAtLocation(member.type.typeName);
  return Boolean(
    imported?.declarations?.some((declaration) => {
      const source = declaration.parent?.parent?.parent?.moduleSpecifier?.text;
      const name = declaration.propertyName?.text ?? declaration.name?.text;
      return (
        typeof source === "string" &&
        options.executorTypeNames.includes(name) &&
        matchesConfiguredModule(source, options.importSpecifier, member.getSourceFile().fileName)
      );
    }),
  );
}

module.exports = { declaredOptionType };
