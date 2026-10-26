"use strict";

const { unwrapTransparentParent } = require("./async-ast");
const { followConst } = require("./async-consumer-property-carrier");

const copyOperators = new Set(["??", "||"]);

// Only the left operand is the supplied callback. The right-hand default is not.
function unwrapCopyLogical(node) {
  const selected = unwrapTransparentParent(node);
  const parent = selected.parent;
  if (
    parent?.type === "LogicalExpression" &&
    copyOperators.has(parent.operator) &&
    parent.left === selected
  )
    return unwrapTransparentParent(parent);
  return selected;
}

function logicalCopyLeft(value) {
  if (value?.type !== "LogicalExpression" || !copyOperators.has(value.operator)) return null;
  return value.left;
}

function plainProperty(entry) {
  return entry.type === "Property" && !entry.computed && !entry.method && entry.kind === "init";
}

// A key before the spread is overwritten by the source object. A key after it
// replaces the copied callback, so that order stays unproven.
function propertyKey(key) {
  return key.type === "Identifier" ? key.name : key.value;
}

function patternBinding(value) {
  const node = value?.type === "AssignmentPattern" ? value.left : value;
  return node?.type === "Identifier" ? node : null;
}

function spreadPreserves(object, spread, property) {
  if (object?.type !== "ObjectExpression") return false;
  const index = object.properties.indexOf(spread);
  const keyOf = (entry) => propertyKey(entry.key);
  return (
    object.properties.slice(0, index).every(plainProperty) &&
    object.properties
      .slice(index + 1)
      .every((entry) => plainProperty(entry) && keyOf(entry) !== property)
  );
}

// Shorthand `const { name }` and `const { name = defaultImpl }` only. A rename,
// rest, or computed key can hide the callback, so those patterns stay unproven.
function destructuredBinding(declarator, property) {
  if (declarator.parent.kind !== "const") return false;
  let binding = null;
  for (const entry of declarator.id.properties) {
    if (entry.type !== "Property" || entry.computed || entry.kind !== "init") return false;
    const name = propertyKey(entry.key);
    const id = patternBinding(entry.value);
    if (id?.name !== name) return false;
    if (name === property) binding = id;
  }
  return binding ?? "safe";
}

// "safe" is a read of some other field. false is an unproven pattern.
function destructureMode(value, property, context, shape, api, seen) {
  const declarator = value.parent;
  if (
    declarator?.type !== "VariableDeclarator" ||
    declarator.init !== value ||
    declarator.id.type !== "ObjectPattern"
  )
    return null;
  const binding = destructuredBinding(declarator, property);
  if (binding === "safe" || binding === false) return binding;
  return followConst(binding, null, context, shape, api, seen);
}

module.exports = { destructureMode, logicalCopyLeft, spreadPreserves, unwrapCopyLogical };
