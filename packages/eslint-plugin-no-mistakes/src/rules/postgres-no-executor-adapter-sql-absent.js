"use strict";

const { unwrapTs } = require("./postgres-query-text");

const stack = [];
let lastOmitted = false;

function noteOmittedOptional() {
  if (stack.length) stack[stack.length - 1] = true;
}

function beginOmittedScope() {
  stack.push(false);
}

function endOmittedScope() {
  const marked = stack.pop() === true;
  if (marked && stack.length) stack[stack.length - 1] = true;
  else if (stack.length === 0) lastOmitted = marked;
}

function consumeOmittedOptional() {
  const marked = lastOmitted;
  lastOmitted = false;
  return marked;
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
  const { visibleSqlTarget } = require("./postgres-no-executor-adapter-visible");
  const proven = visibleSqlTarget(call, index, quoteNode, context, options, accepted, targetFacts);
  return proven === false && consumeOmittedOptional();
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
  beginOmittedScope,
  consumeOmittedOptional,
  endOmittedScope,
  noteOmittedOptional,
  omittedOptionalField,
  reportsOmittedOptional,
};
