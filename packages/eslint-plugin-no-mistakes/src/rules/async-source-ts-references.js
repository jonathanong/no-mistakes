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

// Runtime references skip erased type bindings; type positions keep lexical lookup.
function valueBinding(variable) {
  return Boolean(variable?.defs.some((definition) => definition.type !== "Type"));
}
function lookup(current, name, typeOnly = false) {
  let fallback = null;
  while (current) {
    const variable = current.variables.find((item) => item.name === name);
    if (variable && (typeOnly || valueBinding(variable))) return variable;
    fallback ||= variable;
    current = current.upper;
  }
  return fallback;
}
function scopeViews(owners, typePositions) {
  const views = new WeakMap();
  function view(scope) {
    if (!scope) return null;
    if (!views.has(scope))
      views.set(scope, {
        ...scope,
        upper: view(scope.upper),
        variables: scope.variables.filter(
          (variable) => valueBinding(variable) || !valueBinding(lookup(scope.upper, variable.name)),
        ),
      });
    return views.get(scope);
  }
  return (node) => (typePositions.get(node) ? owners.get(node) : view(owners.get(node)));
}

module.exports = { isReference, write, lookup, scopeViews };
