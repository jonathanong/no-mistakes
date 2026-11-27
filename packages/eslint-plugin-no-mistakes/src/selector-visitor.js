"use strict";

const {
  attributeName,
  canonicalAttribute,
  isSelectorAttribute,
  literalString,
  options,
  selectorAttributes,
} = require("./helpers");
const { unwrapExpression } = require("./rules/async-ast");

function selectorAttributeVisitors(context, callback, reportOpaque) {
  const option = options(context);
  const attrs = selectorAttributes(option);
  const canonical = canonicalAttribute(option);
  return {
    JSXAttribute(node) {
      const name = attributeName(node);
      if (name && isSelectorAttribute(name, attrs)) {
        callback(node, name);
        return;
      }
      if (reportOpaque && namespacedSelector(node, attrs)) {
        reportOpaque(node, { kind: "namespaced" });
      }
    },
    JSXSpreadAttribute(node) {
      if (!reportOpaque) return;
      const shape = spreadSelectorShape(node, attrs, canonical);
      if (shape) reportOpaque(node, shape);
    },
  };
}

function namespacedSelector(attribute, attrs) {
  const name = attribute?.name;
  if (name?.type !== "JSXNamespacedName") return false;
  const local = name.name?.name;
  return typeof local === "string" && attrs.includes(local);
}

function propertyName(prop) {
  if (prop.computed) return literalString(prop.key);
  if (prop.key?.type === "Identifier") return prop.key.name;
  return literalString(prop.key);
}

function spreadSelectorShape(node, attrs, canonical) {
  const arg = unwrapExpression(node.argument);
  if (arg?.type !== "ObjectExpression") return { kind: "opaque" };
  let nonCanonical = false;
  let dynamicValue = false;
  let sawSelector = false;
  for (const prop of arg.properties) {
    if (prop.type !== "Property") return { kind: "opaque" };
    const key = propertyName(prop);
    if (!key) return { kind: "opaque" };
    if (!attrs.includes(key)) continue;
    sawSelector = true;
    if (key !== canonical) nonCanonical = true;
    if (literalString(prop.value) === null) dynamicValue = true;
  }
  return sawSelector ? { kind: "selector", nonCanonical, dynamicValue } : null;
}

module.exports = {
  selectorAttributeVisitors,
};
