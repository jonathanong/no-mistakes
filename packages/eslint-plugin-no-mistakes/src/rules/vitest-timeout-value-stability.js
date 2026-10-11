"use strict";
const { resolveVariable } = require("./async-target-bindings");
const { unwrapTransparentParent } = require("./async-ast");
const { configCall, overrideCall, invocationArguments } = require("./vitest-timeout-bindings");

// Borrow ESLint's existing variable references; do not build another binding graph.
function createValueStability(context, primitiveMemberRead) {
  const memo = new WeakMap();
  const active = new Set();
  function stable(variable) {
    if (!variable || active.has(variable)) return false;
    if (memo.has(variable)) return memo.get(variable);
    active.add(variable);
    const result =
      variable.defs.every(
        (def) =>
          def.type === "Variable" &&
          def.parent.kind === "const" &&
          def.parent.parent?.type !== "ExportNamedDeclaration" &&
          def.parent.parent?.type !== "ExportDefaultDeclaration",
      ) &&
      variable.references.every(
        (reference) => reference.init || (!reference.isWrite() && safeUse(reference.identifier)),
      );
    active.delete(variable);
    memo.set(variable, result);
    return result;
  }
  function safeUse(identifier) {
    let node = unwrapTransparentParent(identifier);
    while (node.parent?.type === "MemberExpression" && node.parent.object === node)
      node = unwrapTransparentParent(node.parent);
    const memberRead = node.type === "MemberExpression" && primitiveMemberRead(node);
    let parent = node.parent;
    if (
      (parent?.type === "AssignmentExpression" && parent.left === node) ||
      (parent?.type === "UpdateExpression" && parent.argument === node) ||
      (parent?.type === "UnaryExpression" && parent.operator === "delete")
    )
      return false;
    // A proven scalar read cannot expose the receiver, but writes above still
    // invalidate it. Object-valued and unresolved properties remain conservative.
    if (memberRead) return true;
    // Container copies can preserve nested references. Follow their existing binding too.
    while (
      parent &&
      ((parent.type === "Property" && parent.value === node) ||
        parent.type === "ObjectExpression" ||
        parent.type === "ArrayExpression" ||
        parent.type === "SpreadElement")
    ) {
      node = parent;
      parent = node.parent;
    }
    if (parent?.type === "VariableDeclarator" && parent.init === node)
      return parent.id.type === "Identifier" && stable(resolveVariable(parent.id, context));
    if (parent?.type === "CallExpression") {
      if (parent.callee === node) return false;
      // Public config/registration consumers are admitted; unknown helpers can mutate objects.
      const config = configCall(parent, context);
      if (config)
        return (
          parent.arguments.indexOf(node) === 0 ||
          (config === "mergeConfig" && parent.arguments.indexOf(node) === 1)
        );
      const args = invocationArguments(parent, context);
      const index = args.indexOf(node);
      const kind = overrideCall(parent, context);
      return (
        (kind === "runtime" && index === 0) ||
        (kind === "hook" && index === 1) ||
        (kind === "test" && (index === 1 || index === 2))
      );
    }
    if (parent?.type === "ReturnStatement" || parent?.type === "AssignmentExpression") return false;
    // Exports and unproved contexts (await/yield, tagged templates, computed
    // keys, and other consumers) can expose the object. No default safe escape.
    return false;
  }
  return stable;
}
module.exports = { createValueStability };
