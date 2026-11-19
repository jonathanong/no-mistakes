"use strict";

const { acceptsExecutor, localFunction } = require("./postgres-no-executor-adapter-target");
const { visibleSqlTarget } = require("./postgres-no-executor-adapter-visible");

function declarationOf(call, context, facts) {
  const services = context.sourceCode.parserServices;
  if (!services?.program || !services.esTreeNodeToTSNodeMap) return null;
  const fn = localFunction(call.callee, context, facts);
  if (fn) return services.esTreeNodeToTSNodeMap.get(fn);
  // Accepted and not local means an imported helper with one resolved signature.
  const callee = services.esTreeNodeToTSNodeMap.get(call.callee);
  const [signature] = services.program
    .getTypeChecker()
    .getTypeAtLocation(callee)
    .getCallSignatures();
  return signature.getDeclaration();
}

// The executor reaches an optional defaulted options parameter, or the callee
// passes the options on. Acceptance there rests on the options contract alone,
// so the SQL the executor receives is proven separately
// (docs/eslint-rules/postgres-no-executor-adapter.md).
function suppliedOptions(call, index, quoteNode, context, facts) {
  if (quoteNode?.type !== "Property") return false;
  const declaration = declarationOf(call, context, facts);
  const parameter = declaration?.parameters?.[index];
  return Boolean(
    ((parameter?.initializer || parameter?.questionToken) &&
      typeof parameter.name.escapedText === "string") ||
    facts.get(declaration)?.forwarded,
  );
}

// The one acceptance every mode goes through: the options contract, then, for a
// supplied executor, the same SQL proof forwarding-only mode applies.
function acceptedTarget(call, index, quoteNode, context, options, facts, mode) {
  if (!acceptsExecutor(call, index, quoteNode, context, options, facts)) return false;
  if (mode !== "all" || !suppliedOptions(call, index, quoteNode, context, facts)) return true;
  return visibleSqlTarget(call, index, quoteNode, context, options, true, facts) === true;
}

module.exports = { acceptedTarget };
