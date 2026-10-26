"use strict";
const { unwrapExpression } = require("./async-ast");

// A slot contract cannot adopt a target promise discarded by an opaque call on
// its result. Only a direct configured target call establishes this handoff.
function targetReturns(fn, context, proof) {
  function direct(value) {
    value = unwrapExpression(value);
    if (value?.type !== "CallExpression" || value.optional) return false;
    if (proof.target(value.callee, context)) return true;
    const type = proof.valueType(value.callee, context);
    return Boolean(type && proof.targetType(type.type, type.context));
  }
  if (fn.body.type !== "BlockStatement") return direct(fn.body);
  let found = false;
  let valid = true;
  function visit(node) {
    if (!node?.type || !valid) return;
    if (
      ["FunctionDeclaration", "FunctionExpression", "ArrowFunctionExpression"].includes(node.type)
    )
      return;
    if (node.type === "ReturnStatement") {
      found = true;
      valid = direct(node.argument);
      return;
    }
    for (const key of context.sourceCode.visitorKeys[node.type] || []) {
      const child = node[key];
      if (Array.isArray(child)) child.forEach(visit);
      else visit(child);
    }
  }
  visit(fn.body);
  return found && valid;
}

module.exports = { targetReturns };
