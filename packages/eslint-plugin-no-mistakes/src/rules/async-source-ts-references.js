"use strict";

function isReference(node, bindings) {
  if (bindings.has(node)) return false;
  const parent = node.parent;
  if (
    ["MemberExpression", "TSQualifiedName"].includes(parent.type) &&
    (parent.property === node || parent.right === node) &&
    !parent.computed
  )
    return false;
  if (
    ["Property", "TSPropertySignature", "TSMethodSignature"].includes(parent.type) &&
    parent.key === node &&
    !parent.computed
  )
    return false;
  if (parent.type.startsWith("Import")) return false;
  if (parent.type === "ExportSpecifier") return parent.local === node && !parent.parent.source;
  return true;
}
function write(node) {
  let value = node;
  let parent = value.parent;
  while (
    parent &&
    ([
      "ObjectPattern",
      "ArrayPattern",
      "ObjectExpression",
      "ArrayExpression",
      "RestElement",
      "SpreadElement",
    ].includes(parent.type) ||
      (parent.type === "Property" && parent.value === value) ||
      (parent.type === "AssignmentPattern" && parent.left === value) ||
      (["TSAsExpression", "TSTypeAssertion", "TSNonNullExpression"].includes(parent.type) &&
        parent.expression === value))
  ) {
    value = parent;
    parent = value.parent;
  }
  return Boolean(
    parent &&
    ((["AssignmentExpression", "ForOfStatement", "ForInStatement"].includes(parent.type) &&
      parent.left === value) ||
      (parent.type === "UpdateExpression" && parent.argument === value)),
  );
}

module.exports = { isReference, write };
