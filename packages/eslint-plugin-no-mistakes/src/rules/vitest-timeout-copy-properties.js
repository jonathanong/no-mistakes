"use strict";
const { resolveVariable } = require("./async-target-bindings");
const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { staticPropertyName } = require("./vitest-timeout-bindings");
const { memberName } = require("./vitest-timeout-copy-paths");

// Assignment can invoke an own or inherited setter. Prove a data descriptor
// from syntax without evaluating the source whose stability is being checked.
function createOwnDataProperty(context) {
  function deleted(variable, seen = new Set()) {
    if (!variable || seen.has(variable)) return false;
    seen = new Set(seen).add(variable);
    return variable.references.some(({ identifier }) => {
      let use = unwrapTransparentParent(identifier);
      while (use.parent?.type === "MemberExpression" && use.parent.object === use)
        use = unwrapTransparentParent(use.parent);
      let parent = use.parent;
      if (parent?.type === "UnaryExpression" && parent.operator === "delete") return true;
      while (
        parent &&
        ((parent.type === "Property" && parent.value === use) ||
          parent.type === "ObjectExpression" ||
          parent.type === "ArrayExpression" ||
          parent.type === "SpreadElement")
      ) {
        use = parent;
        parent = use.parent;
      }
      return (
        parent?.type === "VariableDeclarator" &&
        parent.init === use &&
        parent.id.type === "Identifier" &&
        deleted(resolveVariable(parent.id, context), seen)
      );
    });
  }
  function initializer(node, seen) {
    node = unwrapExpression(node);
    if (node.type === "Identifier") {
      const variable = resolveVariable(node, context);
      const def = variable?.defs[0];
      // Deletion through an alias can expose an inherited setter later.
      if (deleted(variable)) return null;
      return def?.type === "Variable" &&
        def.parent.kind === "const" &&
        def.node.id.type === "Identifier"
        ? def.node.init
        : null;
    }
    if (node.type === "MemberExpression")
      return property(node.object, memberName(node, context), seen)?.value ?? null;
    return node;
  }
  function property(node, key, seen = new Set()) {
    if (!node || key === null || seen.has(node)) return null;
    seen = new Set(seen).add(node);
    const value = initializer(node, seen);
    if (!value) return null;
    if (value !== node) return property(value, key, seen);
    if (node.type === "ArrayExpression") {
      if (key === "length") return { data: true, value: null };
      const entries = arrayValues(node, new Set());
      const element = entries?.find((entry, index) => String(index) === key && entry);
      return element ? { data: true, value: element } : null;
    }
    if (node.type !== "ObjectExpression") return null;
    let uncertainValue = false;
    for (let index = node.properties.length - 1; index >= 0; index--) {
      const entry = node.properties[index];
      if (entry.type === "SpreadElement") {
        const copied = property(entry.argument, key, seen);
        // Spreading reads accessors and creates ordinary own data properties.
        if (copied)
          return { data: true, value: copied.data && !uncertainValue ? copied.value : null };
        // An opaque spread might replace an earlier accessor with a data property,
        // but cannot establish that the accessor was definitely replaced.
        uncertainValue = true;
        continue;
      }
      const name = staticPropertyName(entry, context);
      if (name === null) return null;
      if (name === key) {
        // The special literal prototype property is not an own data descriptor.
        if (key === "__proto__" && !entry.computed && !entry.shorthand && entry.kind === "init")
          return null;
        return { data: entry.kind === "init", value: uncertainValue ? null : entry.value };
      }
    }
    return null;
  }
  function arrayValues(node, seen) {
    if (!node || seen.has(node)) return null;
    seen = new Set(seen).add(node);
    const value = initializer(node, seen);
    if (!value) return null;
    if (value !== node) return arrayValues(value, seen);
    if (node.type !== "ArrayExpression") return null;
    const entries = [];
    for (const element of node.elements) {
      if (element?.type === "SpreadElement") {
        const spread = arrayValues(element.argument, seen);
        if (!spread) return null;
        entries.push(...spread);
      } else entries.push(element);
    }
    return entries;
  }
  return (member) => {
    member = unwrapExpression(member);
    return property(member.object, memberName(member, context))?.data === true;
  };
}
module.exports = { createOwnDataProperty };
