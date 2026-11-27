"use strict";

const { unwrapExpression } = require("./async-ast");
const { isReassigned, resolveVariable } = require("./async-target-bindings");
const { literalString } = require("./module-mock-helpers");

function createStaticName(context) {
  function name(node, seen = new Set()) {
    node = unwrapExpression(node);
    // for-of, for-in, and pattern bindings have no initializer.
    if (!node) return null;
    const literal = literalString(node);
    if (literal !== null) return literal;
    if (node.type === "BinaryExpression" && node.operator === "+") {
      const left = name(node.left, seen);
      const right = name(node.right, seen);
      return left === null || right === null ? null : left + right;
    }
    if (node.type !== "Identifier") return null;
    const variable = resolveVariable(node, context);
    const def = variable?.defs[0];
    if (
      seen.has(variable) ||
      def?.type !== "Variable" ||
      def.parent.kind !== "const" ||
      isReassigned(node, context)
    ) {
      return null;
    }
    return name(def.node.init, new Set([...seen, variable]));
  }
  return name;
}

module.exports = { createStaticName };
