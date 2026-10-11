"use strict";
const { resolveVariable } = require("./async-target-bindings");
const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { staticPropertyName } = require("./vitest-timeout-bindings");

// Probe only the copied property: evaluating the whole spread would re-enter
// the source whose stability is currently being checked.
function createScalarCopyWrite(context, evaluate) {
  function scalar(expression) {
    if (
      ["ArrowFunctionExpression", "FunctionExpression"].includes(unwrapExpression(expression).type)
    )
      return false;
    const value = evaluate(expression);
    return !value.runtimeFunction && (value.kind === "number" || value.kind === "null");
  }
  function knownNull(node, active = new Set()) {
    node = unwrapExpression(node);
    if (node?.type === "Literal" && node.value === null) return true;
    if (node?.type !== "Identifier") return false;
    const variable = resolveVariable(node, context);
    const def = variable?.defs[0];
    if (
      active.has(variable) ||
      def?.type !== "Variable" ||
      def.parent.kind !== "const" ||
      def.node.id.type !== "Identifier" ||
      variable.references.some((reference) => reference.isWrite() && !reference.init)
    )
      return false;
    active.add(variable);
    const result = knownNull(def.node.init, active);
    active.delete(variable);
    return result;
  }
  function property(node, name, active) {
    node = unwrapExpression(node);
    if (node.type === "Identifier") {
      const variable = resolveVariable(node, context);
      const def = variable?.defs[0];
      if (
        active.has(variable) ||
        def?.type !== "Variable" ||
        def.parent.kind !== "const" ||
        def.node.id.type !== "Identifier" ||
        variable.references.some((reference) => reference.isWrite() && !reference.init)
      )
        return null;
      active.add(variable);
      const result = property(def.node.init, name, active);
      active.delete(variable);
      return result;
    }
    if (node.type !== "ObjectExpression") return null;
    for (let index = node.properties.length - 1; index >= 0; index--) {
      const entry = node.properties[index];
      if (entry.type === "SpreadElement") {
        if (knownNull(entry.argument)) continue;
        const result = property(entry.argument, name, active);
        if (result !== undefined) return result;
      } else {
        const key = staticPropertyName(entry, context);
        if (key === null) return null;
        if (key === name) return entry.kind === "init" ? entry.value : null;
      }
    }
    return undefined;
  }
  return (node) => {
    node = unwrapExpression(node);
    const name = staticPropertyName({ key: node.property, computed: node.computed }, context);
    if (name === null) return false;
    const expression = property(node.object, name, new Set());
    if (!expression) return false;
    if (!scalar(expression)) return false;
    // A later coercion must not rely on an initializer overwritten with an object.
    return resolveVariable(unwrapExpression(node.object), context).references.every((reference) => {
      if (reference.init) return true;
      const member = unwrapTransparentParent(reference.identifier).parent;
      if (member?.type !== "MemberExpression") return false;
      const assignment = unwrapTransparentParent(member).parent;
      if (
        assignment?.type !== "AssignmentExpression" ||
        unwrapExpression(assignment.left) !== member
      )
        return true;
      if (!["=", "&&=", "||=", "??="].includes(assignment.operator)) return true;
      const key = staticPropertyName({ key: member.property, computed: member.computed }, context);
      return key !== null && (key !== name || scalar(assignment.right));
    });
  };
}
module.exports = { createScalarCopyWrite };
