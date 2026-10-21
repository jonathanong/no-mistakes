"use strict";

const { literalString } = require("./module-mock-helpers");
const { resolveVariable } = require("./test-no-shared-state-aliases");

const EXPECT_MODULES = new Set(["vitest", "@jest/globals"]);

function propertyName(node) {
  if (!node) return null;
  if (node.type === "Identifier" || (typeof node.name === "string" && node.type == null)) {
    return node.name;
  }
  if (node.type === "Literal") return String(node.value);
  return literalString(node);
}

function memberName(member) {
  if (member?.type !== "MemberExpression") return null;
  if (member.computed && member.property?.type === "Identifier") return null;
  return propertyName(member.property);
}

function unwrap(node) {
  let current = node;
  while (
    current?.type === "ChainExpression" ||
    current?.type === "TSAsExpression" ||
    current?.type === "TSTypeAssertion" ||
    current?.type === "TSNonNullExpression"
  ) {
    current = current.expression;
  }
  if (current?.type === "SequenceExpression" && current.expressions?.length) {
    return unwrap(current.expressions.at(-1));
  }
  return current;
}

function isExpectBinding(def) {
  const imported = def.node?.imported;
  const importedName = imported?.type === "Literal" ? String(imported.value) : imported?.name;
  return (
    def.type === "ImportBinding" &&
    EXPECT_MODULES.has(def.parent?.source?.value) &&
    importedName === "expect"
  );
}

function isNamespace(node, context, seen) {
  if (node?.type !== "Identifier" || seen.has(node.name)) return false;
  const variable = resolveVariable(node, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (
    def?.type === "ImportBinding" &&
    def.node.type === "ImportNamespaceSpecifier" &&
    EXPECT_MODULES.has(def.parent?.source?.value)
  ) {
    return true;
  }
  if (
    def?.type !== "Variable" ||
    def.node?.parent?.kind !== "const" ||
    def.node.id.type !== "Identifier"
  ) {
    return false;
  }
  seen.add(node.name);
  return isNamespace(def.node.init, context, seen);
}

function isExpectCallee(node, context, seen = new Set()) {
  let callee = unwrap(node);
  if (callee?.type === "MemberExpression") {
    const name = memberName(callee);
    if (!callee.computed && (name === "call" || name === "apply")) {
      return isExpectCallee(callee.object, context, seen);
    }
    if (name === "soft" || name === "poll") return isExpectCallee(callee.object, context, seen);
    return name === "expect" && isNamespace(callee.object, context, seen);
  }
  if (callee?.type !== "Identifier" || seen.has(callee.name)) return false;
  const variable = resolveVariable(callee, context);
  if (!variable || variable.defs.length === 0) return callee.name === "expect";
  if (variable.defs.every(isExpectBinding)) return true;
  const def = variable.defs.length === 1 ? variable.defs[0] : null;
  if (def?.type !== "Variable" || def.node?.parent?.kind !== "const" || !def.node.init)
    return false;
  seen.add(callee.name);
  if (def.node.id.type === "Identifier") return isExpectCallee(def.node.init, context, seen);
  if (def.node.id.type !== "ObjectPattern" || !isNamespace(def.node.init, context, seen)) {
    return false;
  }
  return def.node.id.properties.some((property) => {
    let value = property.value;
    while (value?.type === "AssignmentPattern") value = value.left;
    const key = property.computed ? literalString(property.key) : propertyName(property.key);
    return value?.name === callee.name && key === "expect";
  });
}

function expectArguments(node, context) {
  const current = unwrap(node);
  if (current?.type !== "CallExpression") return null;
  let callee = unwrap(current.callee);
  let args = current.arguments;
  if (callee?.type === "MemberExpression" && !callee.computed) {
    const name = memberName(callee);
    if (name === "call") {
      callee = callee.object;
      args = args.slice(1);
    } else if (name === "apply" && args[1]?.type === "ArrayExpression") {
      callee = callee.object;
      args = args[1].elements;
    }
  }
  return isExpectCallee(callee, context) ? args : null;
}

function isExpectCall(node, context) {
  return expectArguments(node, context) !== null;
}

module.exports = {
  expectArguments,
  isExpectCall,
  isExpectCallee,
  isNamespace,
  memberName,
  propertyName,
  unwrap,
};
