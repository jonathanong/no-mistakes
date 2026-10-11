"use strict";
const { resolveVariable } = require("./async-target-bindings");
const { unwrapExpression } = require("./async-ast");
const { pathStartsWith } = require("./vitest-timeout-copy-paths");

function createCopySlotWrite(context, ownDataProperty) {
  function safeReceiver(node, deletion, seen = new Set()) {
    node = unwrapExpression(node);
    if (node.type === "Identifier") {
      const variable = resolveVariable(node, context);
      const def = variable?.defs[0];
      if (
        seen.has(variable) ||
        def?.type !== "Variable" ||
        def.parent.kind !== "const" ||
        def.node.id.type !== "Identifier" ||
        !def.node.init ||
        variable.references.some((reference) => reference.isWrite() && !reference.init)
      )
        return false;
      return safeReceiver(def.node.init, deletion, new Set(seen).add(variable));
    }
    if (node.type === "MemberExpression")
      // Only this delete is ignored. A different deletion can expose a getter
      // through the prototype before this receiver is evaluated.
      return ownDataProperty(node, deletion) && safeReceiver(node.object, deletion, seen);
    return node.type === "ObjectExpression" || node.type === "ArrayExpression";
  }
  return (member, operation, members, copyPath) => {
    if (!members.length || members.length > copyPath.length || !pathStartsWith(copyPath, members))
      return false;
    member = unwrapExpression(member);
    // Discarding an own copy slot or wrapper ancestor does not mutate its source.
    if (operation.type === "AssignmentExpression")
      return operation.operator === "=" && ownDataProperty(member);
    return operation.type === "UnaryExpression" && safeReceiver(member.object, operation);
  };
}
module.exports = { createCopySlotWrite };
