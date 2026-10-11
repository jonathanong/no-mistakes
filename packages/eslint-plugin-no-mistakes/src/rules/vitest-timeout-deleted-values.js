"use strict";
const { resolveVariable } = require("./async-target-bindings");
const { unwrapTransparentParent } = require("./async-ast");

// Own slots detach at a shallow spread; nested receiver identities remain shared.
function createDeletedValue(context) {
  function deleted(variable, depth = 0, ignoredDeletion, seen = new Set()) {
    if (!variable || seen.has(variable)) return false;
    seen = new Set(seen).add(variable);
    return variable.references.some(({ identifier }) => {
      let use = unwrapTransparentParent(identifier);
      let members = 0;
      while (use.parent?.type === "MemberExpression" && use.parent.object === use) {
        members++;
        use = unwrapTransparentParent(use.parent);
      }
      let parent = use.parent;
      if (parent?.type === "UnaryExpression" && parent.operator === "delete")
        return members > depth && parent !== ignoredDeletion;
      if (members > depth) return false;
      let nested = depth - members;
      while (
        parent &&
        ((parent.type === "Property" && parent.value === use) ||
          parent.type === "ObjectExpression" ||
          parent.type === "ArrayExpression" ||
          parent.type === "SpreadElement")
      ) {
        if (parent.type === "SpreadElement" && nested === 0) return false;
        if (
          parent.type === "Property" ||
          (parent.type === "ArrayExpression" && use.type !== "SpreadElement")
        )
          nested++;
        use = unwrapTransparentParent(parent);
        parent = use.parent;
      }
      return (
        parent?.type === "VariableDeclarator" &&
        parent.init === use &&
        parent.id.type === "Identifier" &&
        deleted(resolveVariable(parent.id, context), nested, ignoredDeletion, seen)
      );
    });
  }
  return deleted;
}
module.exports = { createDeletedValue };
