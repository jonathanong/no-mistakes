"use strict";
const { unwrapSequence } = require("./async-ast");
const { isReassigned, memberPropertyName, resolveVariable } = require("./async-target-bindings");
const { staticPropertyName } = require("./imported-call-options");

function member(object, name) {
  return {
    type: "MemberExpression",
    object,
    computed: false,
    property: { type: "Identifier", name },
  };
}

// Keep original identifier nodes so global binding checks use their lexical scope.
function expandSelectorAlias(node, context, preserve, seen = new Set(), asReceiver = false) {
  node = unwrapSequence(node);
  if (preserve(node, asReceiver)) return node;
  if (node?.type === "MemberExpression") {
    if (!node.computed && node.property.type !== "Identifier") return null;
    const name = memberPropertyName(node);
    const object = name && expandSelectorAlias(node.object, context, preserve, seen, true);
    return object ? member(object, name) : null;
  }
  if (node?.type !== "Identifier") return null;
  const variable = resolveVariable(node, context);
  if (!variable) return node;
  if (seen.has(variable) || isReassigned(node, context)) return null;
  if (variable.defs.length !== 1 || variable.defs[0].type !== "Variable") return node;
  const def = variable.defs[0];
  if (def.parent?.kind !== "const" || !def.node.init) return null;
  seen.add(variable);
  if (def.node.id.type === "Identifier")
    return expandSelectorAlias(def.node.init, context, preserve, seen, asReceiver);
  if (def.node.id.type !== "ObjectPattern") return null;
  const property = def.node.id.properties.find(
    (item) =>
      item.type === "Property" && item.value.type === "Identifier" && item.value.name === node.name,
  );
  if (!property) return null;
  const name = staticPropertyName(property);
  const object = name && expandSelectorAlias(def.node.init, context, preserve, seen, true);
  return object ? member(object, name) : null;
}
module.exports = { expandSelectorAlias };
