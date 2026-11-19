"use strict";

const { unwrapTs } = require("./postgres-query-text");

// An object literal argument that omits `field` reads it as undefined, which is
// non-row data only when the parameter type declares that field optional
// (docs/eslint-rules/postgres-no-executor-adapter.md).
function omittedOptionalField(node, field, context) {
  node = unwrapTs(node);
  const services = context.sourceCode.parserServices;
  if (node.type !== "ObjectExpression" || !services?.program?.getTypeChecker) return false;
  const ts = require("typescript");
  const checker = services.program.getTypeChecker();
  const contextual = checker.getContextualType(services.esTreeNodeToTSNodeMap.get(node));
  const type = contextual?.getNonNullableType();
  const parts = type?.isUnion() ? type.types : [type];
  return parts.every((part) => Boolean(part?.getProperty(field)?.flags & ts.SymbolFlags.Optional));
}

module.exports = { omittedOptionalField };
