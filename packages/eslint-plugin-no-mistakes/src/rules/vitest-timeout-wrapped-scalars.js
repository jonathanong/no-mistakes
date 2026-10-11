"use strict";
const { resolveVariable } = require("./async-target-bindings");
const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { staticPropertyName } = require("./vitest-timeout-bindings");
const { memberName } = require("./vitest-timeout-copy-paths");

function memberPath(node, parent = false) {
  const keys = [];
  node = parent ? unwrapTransparentParent(node) : unwrapExpression(node);
  if (parent) {
    while (node.parent?.type === "MemberExpression" && node.parent.object === node) {
      keys.push(node.parent);
      node = unwrapTransparentParent(node.parent);
    }
  } else {
    while (node.type === "MemberExpression") {
      keys.unshift(node);
      node = unwrapExpression(node.object);
    }
  }
  return { node, keys };
}
function wrapperProperty(node, name, context) {
  node = unwrapExpression(node);
  if (node.type === "ArrayExpression") {
    if (node.elements.some((entry) => entry?.type === "SpreadElement")) return null;
    return node.elements[Number(name)];
  }
  if (node.type !== "ObjectExpression") return null;
  // Getters and prototype setters can mutate a sibling through `this` without
  // producing a reference to the wrapper's binding. Keep wrappers literal and plain.
  if (
    node.properties.some(
      (entry) =>
        entry.type !== "Property" ||
        entry.kind !== "init" ||
        staticPropertyName(entry, context) === "__proto__",
    )
  )
    return null;
  for (let index = node.properties.length - 1; index >= 0; index--) {
    const entry = node.properties[index];
    const key = staticPropertyName(entry, context);
    if (key === null) return null;
    if (key === name) return entry.value;
  }
  return null;
}
function createWrappedScalarWrite(context, scalar, property) {
  return (node, name) => {
    const receiver = memberPath(node.object);
    // Inline wrapper expressions have no binding whose uses can be checked.
    if (receiver.node.type !== "Identifier") return false;
    const variable = resolveVariable(receiver.node, context);
    let projected = variable.defs[0].node.init;
    const path = receiver.keys.map((member) => memberName(member, context));
    for (const key of path) {
      projected = projected && wrapperProperty(projected, key, context);
      if (!projected) return false;
    }
    // Identifier-backed projected receivers need an alias-set proof of their own.
    if (unwrapExpression(projected).type !== "ObjectExpression") return false;
    const expression = property(projected, name, new Set(), true);
    if (!expression || !scalar(expression)) return false;
    const target = [...path, name];
    return variable.references.every((reference) => {
      if (reference.init) return true;
      const use = memberPath(reference.identifier, true);
      const keys = use.keys.map((member) => memberName(member, context));
      if (keys.includes(null)) return false;
      const prefix = keys.every((key, index) => target[index] === key);
      // A projected receiver alias can receive writes invisible on the original
      // binding. Prefix replacement or escape also invalidates its initializer.
      if (prefix && keys.length < target.length) return false;
      const assignment = use.node.parent;
      if (
        assignment?.type !== "AssignmentExpression" ||
        unwrapExpression(assignment.left) !== unwrapExpression(use.node) ||
        !["=", "&&=", "||=", "??="].includes(assignment.operator)
      )
        return true;
      return !prefix || scalar(assignment.right);
    });
  };
}
module.exports = { createWrappedScalarWrite };
