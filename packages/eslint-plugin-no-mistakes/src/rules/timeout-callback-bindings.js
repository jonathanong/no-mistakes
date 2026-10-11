"use strict";
const { resolveVariable, isReassigned } = require("./async-target-bindings");
const { unwrapExpression, isFunction } = require("./async-ast");
const { literalString } = require("./module-mock-helpers");
function createCallbackBindings(context) {
  function callback(node, seen = new Set(), allowPartial = false) {
    node = unwrapExpression(node);
    if (isFunction(node)) return node;
    if (
      node?.type === "CallExpression" &&
      node.callee.type === "MemberExpression" &&
      member(node.callee) === "bind"
    ) {
      const partial =
        node.arguments.length > 1 || node.arguments.some((arg) => arg.type === "SpreadElement");
      return partial && !allowPartial ? null : callback(node.callee.object, seen, allowPartial);
    }
    if (node?.type !== "Identifier") return null;
    const variable = resolveVariable(node, context),
      def = variable?.defs[0];
    if (seen.has(variable)) return null;
    if (def?.type === "FunctionName")
      return variable.references.some((ref) => ref.isWrite() && !ref.init) ? null : def.node;
    if (isReassigned(node, context)) return null;
    if (def?.type !== "Variable" || def.parent.kind !== "const") return null;
    return callback(def.node.init, new Set([...seen, variable]), allowPartial);
  }
  function staticName(node, seen = new Set()) {
    node = unwrapExpression(node);
    const literal = literalString(node);
    if (literal !== null) return literal;
    if (node?.type === "BinaryExpression" && node.operator === "+") {
      const left = staticName(node.left, seen),
        right = staticName(node.right, seen);
      return left === null || right === null ? null : left + right;
    }
    if (node?.type !== "Identifier") return null;
    const variable = resolveVariable(node, context),
      def = variable?.defs[0];
    if (
      seen.has(variable) ||
      def?.type !== "Variable" ||
      def.parent.kind !== "const" ||
      isReassigned(node, context)
    )
      return null;
    return staticName(def.node.init, new Set([...seen, variable]));
  }
  const member = (node) => (node.computed ? staticName(node.property) : node.property.name);
  return { callback, staticName, member };
}
module.exports = { createCallbackBindings };
