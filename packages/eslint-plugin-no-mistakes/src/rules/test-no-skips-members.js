"use strict";

const { stableConstInitializer } = require("./async-target-bindings");
const { requireResolution } = require("./async-target-sources");
const { literalString } = require("./module-mock-helpers");
const { unwrapExpression } = require("./async-ast");

function immutableKey(node, context) {
  const seen = new Set();
  while (node) {
    node = unwrapExpression(node);
    const literal = literalString(node);
    if (literal !== null) return literal;
    node = stableConstInitializer(node, context, seen);
  }
  return null;
}

function memberName(node, context) {
  if (!node.computed && node.property.type === "Identifier") return node.property.name;
  return immutableKey(node.property, context);
}

function unwrapBinding(node) {
  let current = node;
  while (current.type === "AssignmentPattern") current = current.left;
  return current;
}

function propertyKey(property) {
  if (!property.computed && property.key.type === "Identifier") return property.key.name;
  return literalString(property.key);
}

// The key that introduces the identifier, after defaults and nested objects.
// `const { test: t = x } = require("vitest")` and
// `const { runner: { test: t } = fallback } = require("vitest")` both yield `test`.
function patternBindingName(pattern, name) {
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const value = unwrapBinding(property.value);
    if (value.type === "Identifier" && value.name === name) return propertyKey(property);
    if (value.type === "ObjectPattern") {
      const nested = patternBindingName(value, name);
      if (nested) return nested;
    }
  }
  return null;
}

// `const check = require("vitest").test` is the named export, not the module.
function requireExport(init, context) {
  if (init?.type !== "MemberExpression") return null;
  const name = memberName(init, context);
  if (!name) return null;
  const required = requireResolution(unwrapExpression(init.object), context);
  return required ? { source: required.source, name } : null;
}

function requireMemberBinding(object, name, context, admit, testModules) {
  if (!name) return null;
  const required = requireResolution(unwrapExpression(object), context);
  if (!required?.source) return null;
  const admitted = admit(required.source, name, false, testModules);
  return admitted?.framework ? admitted : null;
}

module.exports = { memberName, patternBindingName, requireExport, requireMemberBinding };
