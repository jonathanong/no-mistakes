"use strict";

const { unwrapTs } = require("./postgres-query-text");

function noteOmittedOptional(result) {
  if (result) result.omitted = true;
}

function reportsOmittedOptional(
  mode,
  nested,
  accepted,
  call,
  index,
  quoteNode,
  context,
  options,
  targetFacts,
) {
  if (mode !== "all" || nested || !accepted) return false;
  const result = { omitted: false };
  const { visibleSqlTarget } = require("./postgres-no-executor-adapter-visible");
  const proven = visibleSqlTarget(
    call,
    index,
    quoteNode,
    context,
    options,
    accepted,
    targetFacts,
    undefined,
    result,
  );
  return proven === false && result.omitted;
}

// True when an object literal omits optional `field` (the read is undefined).
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

module.exports = {
  noteOmittedOptional,
  omittedOptionalField,
  reportsOmittedOptional,
};
