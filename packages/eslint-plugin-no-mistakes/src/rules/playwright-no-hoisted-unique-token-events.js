"use strict";

const { childNodes } = require("./test-no-shared-state-helpers");
const { resolveVariable } = require("./test-no-shared-state-aliases");

// The TypeScript node types that wrap a runtime value expression rather than being purely
// type-positioned; every other "TS"-prefixed node type is skipped entirely by `collectEvents`.
const TS_VALUE_WRAPPER_TYPES = new Set([
  "TSAsExpression",
  "TSSatisfiesExpression",
  "TSNonNullExpression",
  "TSInstantiationExpression",
  "TSTypeAssertion",
]);

// An Identifier assigned to (not read) by a plain `=` assignment is a write, not a hoisted-value
// reference: a hook that reassigns the tracked variable before reading it is no longer reusing the
// hoisted value.
function isWriteTarget(node) {
  const parent = node.parent;
  return parent?.type === "AssignmentExpression" && parent.operator === "=" && parent.left === node;
}

const KEYED_MEMBER_TYPES = new Set(["MethodDefinition", "PropertyDefinition"]);

// Skip Identifier positions that are names, not value references: the non-computed `.property`
// of a member access, a non-computed, non-shorthand object-literal `.key`, a non-computed class
// method/field `.key` (a class declared inside the hook can legally name a member after the
// tracked variable without reading it), and a plain-assignment write target.
function isReferenceIdentifier(node) {
  const parent = node.parent;
  if (!parent) return true;
  if (parent.type === "MemberExpression" && !parent.computed && parent.property === node) {
    return false;
  }
  if (parent.type === "Property" && !parent.computed && !parent.shorthand && parent.key === node) {
    return false;
  }
  if (KEYED_MEMBER_TYPES.has(parent.type) && !parent.computed && parent.key === node) {
    return false;
  }
  return !isWriteTarget(node);
}

// Collects every read and write of an Identifier reachable from `node`, in source order.
// TypeScript type-only positions (`type X = typeof suffix`, `: typeof suffix` annotations, a
// `TSInterfaceDeclaration` body, ...) are skipped entirely — the value-wrapper expressions above
// recurse into their runtime `.expression` only, never a type operand.
function collectEvents(node, results) {
  if (!node || typeof node.type !== "string") return;
  if (node.type.startsWith("TS")) {
    if (TS_VALUE_WRAPPER_TYPES.has(node.type)) collectEvents(node.expression, results);
    return;
  }
  if (node.type === "Identifier") {
    if (isWriteTarget(node)) results.push({ node, isWrite: true });
    else if (isReferenceIdentifier(node)) results.push({ node, isWrite: false });
  }
  for (const child of childNodes(node)) collectEvents(child, results);
}

// A write's own right-hand side evaluates in full before the assignment takes effect, so a
// self-referential write (`suffix = suffix.trim()`) still observes the pre-write, potentially
// stale value — the result carries the same collision hazard as the value it derives from and
// must not be treated as a fresh refresh of the tracked declarator.
function writeReusesTrackedValue(node, declarator, context) {
  const rhsEvents = [];
  collectEvents(node.parent.right, rhsEvents);
  return rhsEvents.some((rhsEvent) => {
    const variable = resolveVariable(rhsEvent.node, context);
    return variable?.defs?.some((def) => def.node === declarator) ?? false;
  });
}

module.exports = { collectEvents, writeReusesTrackedValue };
