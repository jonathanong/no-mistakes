"use strict";
const { resolveVariable } = require("./async-target-bindings");
const { unwrapExpression } = require("./async-ast");
const { createDeletedValue } = require("./vitest-timeout-deleted-values");
const { staticPropertyName } = require("./vitest-timeout-bindings");
const { memberName } = require("./vitest-timeout-copy-paths");

// Assignment can invoke an own or inherited setter. Prove a data descriptor
// from syntax without evaluating the source whose stability is being checked.
function createOwnDataProperty(context) {
  const deleted = createDeletedValue(context);
  function receiverDeleted(node, depth, probe, seen = new Set()) {
    if (!node || seen.has(node)) return false;
    seen = new Set(seen).add(node);
    node = unwrapExpression(node);
    if (node.type === "MemberExpression")
      return receiverDeleted(node.object, depth + 1, probe, seen);
    if (node.type === "Identifier") {
      const variable = resolveVariable(node, context);
      if (deleted(variable, depth, probe)) return true;
      const def = variable?.defs[0];
      return (
        def?.type === "Variable" &&
        def.parent.kind === "const" &&
        def.node.id.type === "Identifier" &&
        receiverDeleted(def.node.init, depth, probe, seen)
      );
    }
    // A wrapper alias can retain children through its own literal initializer.
    if (depth && (node.type === "ObjectExpression" || node.type === "ArrayExpression")) {
      const children = node.type === "ObjectExpression" ? node.properties : node.elements;
      return children.some((child) =>
        child?.type === "SpreadElement"
          ? receiverDeleted(child.argument, depth, probe, seen)
          : receiverDeleted(
              child?.type === "Property" ? child.value : child,
              depth - 1,
              probe,
              seen,
            ),
      );
    }
    return false;
  }
  function initializer(node, seen, probe) {
    node = unwrapExpression(node);
    if (node.type === "Identifier") {
      const variable = resolveVariable(node, context);
      const def = variable?.defs[0];
      // Deletion through an alias can expose an inherited setter later.
      if (deleted(variable, 0, probe)) return null;
      return def?.type === "Variable" &&
        def.parent.kind === "const" &&
        def.node.id.type === "Identifier"
        ? def.node.init
        : null;
    }
    if (node.type === "MemberExpression")
      return receiverDeleted(node, 0, probe)
        ? null
        : (property(node.object, memberName(node, context), seen, probe)?.value ?? null);
    return node;
  }
  function property(node, key, seen = new Set(), probe) {
    if (!node || key === null || seen.has(node)) return null;
    seen = new Set(seen).add(node);
    const value = initializer(node, seen, probe);
    if (!value) return null;
    if (value !== node) {
      const descriptor = property(value, key, seen, probe);
      // A copied wrapper can preserve a child whose own descriptor was deleted.
      return descriptor &&
        node.type === "Identifier" &&
        deleted(resolveVariable(node, context), 1, probe)
        ? { ...descriptor, value: null }
        : descriptor;
    }
    if (node.type === "ArrayExpression") {
      if (key === "length") return { data: true, enumerable: false, value: null };
      const index = Number(key);
      if (!Number.isInteger(index) || index < 0 || String(index) !== key) return null;
      const element = arrayValueAt(node, index, new Set(), probe);
      return element?.found && element.value
        ? { data: true, enumerable: true, value: element.value }
        : null;
    }
    if (node.type !== "ObjectExpression") return null;
    let uncertainValue = false;
    for (let index = node.properties.length - 1; index >= 0; index--) {
      const entry = node.properties[index];
      if (entry.type === "SpreadElement") {
        const copied = property(entry.argument, key, seen, probe);
        // Object spread skips non-enumerable own properties such as array length.
        if (copied?.enumerable === false) continue;
        // Spreading reads accessors and creates ordinary own data properties.
        if (copied)
          return {
            data: true,
            enumerable: true,
            value: copied.data && !uncertainValue ? copied.value : null,
          };
        // An opaque spread might replace an earlier accessor with a data property,
        // but cannot establish that the accessor was definitely replaced.
        uncertainValue = true;
        continue;
      }
      const name = staticPropertyName(entry, context);
      if (name === null) return null;
      if (name === key) {
        // The special literal prototype property is not an own data descriptor.
        if (
          key === "__proto__" &&
          !entry.computed &&
          !entry.shorthand &&
          !entry.method &&
          entry.kind === "init"
        )
          return null;
        return {
          data: entry.kind === "init",
          enumerable: true,
          value: uncertainValue ? null : entry.value,
        };
      }
    }
    return null;
  }
  function arrayValueAt(node, index, seen, probe) {
    if (!node || seen.has(node)) return null;
    seen = new Set(seen).add(node);
    const value = initializer(node, seen, probe);
    if (!value) return null;
    if (value !== node) {
      if (node.type === "Identifier" && deleted(resolveVariable(node, context), 1, probe))
        return null;
      return arrayValueAt(value, index, seen, probe);
    }
    if (node.type !== "ArrayExpression") return null;
    let offset = 0;
    for (const element of node.elements) {
      if (element?.type === "SpreadElement") {
        const spread = arrayValueAt(element.argument, index - offset, seen, probe);
        if (!spread) return null;
        if (spread.found) return spread;
        offset += spread.length;
      } else {
        if (index === offset) return { found: true, value: element };
        offset++;
      }
    }
    return { found: false, length: offset };
  }
  return (member, ignoredDeletion) => {
    member = unwrapExpression(member);
    const probe = { member, ignoredDeletion };
    return property(member.object, memberName(member, context), new Set(), probe)?.data === true;
  };
}
module.exports = { createOwnDataProperty };
