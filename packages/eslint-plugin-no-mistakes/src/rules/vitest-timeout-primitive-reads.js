"use strict";
const { resolveVariable } = require("./async-target-bindings");
const { unwrapExpression } = require("./async-ast");
const { staticPropertyName } = require("./vitest-timeout-bindings");

// Resolve a property from its initializer without evaluating the receiver
// identifier, whose stability is currently being checked by the caller.
function createPrimitiveMemberRead(context, evaluate) {
  function value(node) {
    node = unwrapExpression(node);
    if (node.type === "MemberExpression") {
      const owner = value(node.object);
      const name = staticPropertyName({ key: node.property, computed: node.computed }, context);
      return owner?.kind === "object" ? owner.properties.get(name)?.value : null;
    }
    // Stability only probes reads rooted in the const binding it has validated.
    const def = resolveVariable(node, context).defs[0];
    return evaluate(def.node.init);
  }
  return (node) => {
    const resolved = value(node);
    return resolved?.kind === "number" || resolved?.kind === "null";
  };
}
module.exports = { createPrimitiveMemberRead };
