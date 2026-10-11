"use strict";
const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");

function initializer(variable) {
  const def = variable?.defs[0];
  return variable?.defs.length === 1 &&
    def.type === "Variable" &&
    def.parent.kind === "const" &&
    def.node.id.type === "Identifier" &&
    !["ExportNamedDeclaration", "ExportDefaultDeclaration"].includes(def.parent.parent?.type)
    ? def.node.init
    : null;
}
function immutableArray(variable, context, active = new Set()) {
  if (!initializer(variable) || active.has(variable)) return false;
  active.add(variable);
  const result = variable.references.every((reference) => {
    if (reference.init) return true;
    if (reference.isWrite()) return false;
    const node = unwrapTransparentParent(reference.identifier),
      parent = node.parent;
    // Spreads consume array elements, without exposing the array receiver.
    if (parent?.type === "SpreadElement")
      return ["ArrayExpression", "ObjectExpression"].includes(parent.parent.type);
    return (
      parent?.type === "VariableDeclarator" &&
      parent.init === node &&
      parent.id.type === "Identifier" &&
      immutableArray(resolveVariable(parent.id, context), context, active)
    );
  });
  active.delete(variable);
  return result;
}
function arrayLength(expression, context, active = new Set()) {
  const node = unwrapExpression(expression);
  if (node?.type === "Identifier") {
    const variable = resolveVariable(node, context);
    if (!immutableArray(variable, context) || active.has(variable)) return null;
    active.add(variable);
    const length = arrayLength(initializer(variable), context, active);
    active.delete(variable);
    return length;
  }
  if (node?.type !== "ArrayExpression") return null;
  let length = 0;
  for (const element of node.elements) {
    const added =
      element?.type === "SpreadElement" ? arrayLength(element.argument, context, active) : 1;
    if (added === null) return null;
    length += added;
  }
  return length;
}
module.exports = { arrayLength };
