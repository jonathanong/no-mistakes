"use strict";

const { resolveVariable } = require("./postgres-query-text");
const { memberPropertyName } = require("./postgres-executor");
const {
  TS_ASSERTIONS,
  constInitializer,
  variableIsImportedExecutor,
  variableIsScopedExecutor,
} = require("./postgres-no-executor-adapter-binding");

function broadQueryMembers(options) {
  return !options.importSpecifier && (options.executorNames ?? []).includes("query");
}

// A configured module judges `.query` only on a proven receiver: a configured
// import, a scoped factory or typed parameter, or a const alias of one.
// `executorNames: [query]` with no module is the explicit broad selector.
function provenQueryReceiver(node, context, bindings, options, seen) {
  let current = node;
  while (current && TS_ASSERTIONS.has(current.type)) current = current.expression;
  if (!current || current.type === "ChainExpression" || current.optional) return null;
  if (current.type === "ConditionalExpression") {
    const consequent = provenQueryReceiver(current.consequent, context, bindings, options, seen);
    const alternate = provenQueryReceiver(current.alternate, context, bindings, options, seen);
    return consequent && alternate ? true : null;
  }
  if (current.type !== "Identifier") return null;
  const variable = resolveVariable(current, context);
  if (
    variableIsImportedExecutor(variable, options) ||
    variableIsScopedExecutor(variable, bindings.scoped ?? [], current)
  ) {
    return true;
  }
  const init = constInitializer(current, context, seen);
  return init ? provenQueryReceiver(init, context, bindings, options, seen) : null;
}

function isProvenQueryMember(node, context, bindings, options) {
  if (node?.type !== "MemberExpression" || node.optional) return false;
  if (memberPropertyName(node) !== "query") return false;
  if (broadQueryMembers(options)) return true;
  if (!options.importSpecifier || bindings.queryMembers !== true) return false;
  return provenQueryReceiver(node.object, context, bindings, options, new Set()) === true;
}

module.exports = {
  isProvenQueryMember,
};
