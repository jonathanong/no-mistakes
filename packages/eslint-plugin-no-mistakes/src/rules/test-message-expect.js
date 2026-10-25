"use strict";

const { literalString } = require("./module-mock-helpers");
const { resolveVariable } = require("./test-no-shared-state-aliases");

const EXPECT_MODULES = new Set(["vitest", "@jest/globals"]);
const MESSAGE_MODULES = new Set([...EXPECT_MODULES, "@playwright/test"]);

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
    current?.type === "TSNonNullExpression" ||
    current?.type === "TSSatisfiesExpression"
  ) {
    current = current.expression;
  }
  if (current?.type === "SequenceExpression" && current.expressions?.length) {
    return unwrap(current.expressions.at(-1));
  }
  return current;
}

function isExpectBinding(def, modules) {
  const imported = def.node?.imported;
  const importedName = imported?.type === "Literal" ? String(imported.value) : imported?.name;
  const source = def.parent?.source?.value;
  return (
    def.type === "ImportBinding" && importedName === "expect" && (!modules || modules.has(source))
  );
}

function isNestedBinding(variable) {
  const scope = variable?.scope;
  const scopeType = scope?.type;
  if (!scopeType || scopeType === "module" || scopeType === "global") return false;
  // CommonJS wraps the program in a function scope whose block is the Program.
  return !(scopeType === "function" && scope.block?.type === "Program");
}

function isNamespace(node, context, seen, modules = null) {
  if (node?.type !== "Identifier" || seen.has(node.name)) return false;
  const variable = resolveVariable(node, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (
    def?.type === "ImportBinding" &&
    ((def.node.type === "ImportNamespaceSpecifier" &&
      (modules || MESSAGE_MODULES).has(def.parent?.source?.value)) ||
      (!modules &&
        def.node.type === "ImportDefaultSpecifier" &&
        def.parent?.source?.value === "@playwright/test"))
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
  if (variable.references.some((ref) => ref.isWrite() && !ref.init)) return false;
  seen.add(node.name);
  return isNamespace(def.node.init, context, seen, modules);
}

function isExpectCallee(node, context, seen = new Set(), modules = null, mode = {}) {
  let callee = unwrap(node);
  if (callee?.type === "MemberExpression") {
    const name = memberName(callee);
    if (name === "call" || name === "apply") {
      return (
        mode.allowForwarding !== false &&
        isExpectCallee(callee.object, context, seen, modules, mode)
      );
    }
    if (name === "soft" || (name === "poll" && mode.allowPoll !== false))
      return isExpectCallee(callee.object, context, seen, modules, mode);
    return name === "expect" && isNamespace(callee.object, context, seen, modules);
  }
  if (callee?.type !== "Identifier" || seen.has(callee.name)) return false;
  const variable = resolveVariable(callee, context);
  if (!variable || variable.defs.length === 0) return callee.name === "expect";
  if (variable.defs.length !== 1) return false;
  const def = variable.defs[0];
  if (def.type === "Parameter") return false;
  if (isExpectBinding(def, modules)) return true;
  if (!modules && callee.name === "expect" && !isNestedBinding(variable)) return true;
  if (def.type !== "Variable" || def.node?.parent?.kind !== "const" || !def.node.init) return false;
  seen.add(callee.name);
  if (def.node.id.type === "Identifier")
    return isExpectCallee(def.node.init, context, seen, modules, mode);
  if (def.node.id.type !== "ObjectPattern") return false;
  const fromModule =
    isNamespace(def.node.init, context, seen, modules) ||
    (!modules && requireExpectSource(def.node.init));
  if (!fromModule) return false;
  return def.node.id.properties.some((property) => {
    let value = property.value;
    while (value?.type === "AssignmentPattern") value = value.left;
    const key = property.computed ? literalString(property.key) : propertyName(property.key);
    return value?.name === callee.name && key === "expect";
  });
}

function requireExpectSource(node) {
  const call = unwrap(node);
  if (call?.type !== "CallExpression") return false;
  const callee = unwrap(call.callee);
  if (callee?.type !== "Identifier" || callee.name !== "require") return false;
  return EXPECT_MODULES.has(literalString(call.arguments?.[0]));
}

function expectArguments(node, context) {
  const current = unwrap(node);
  if (current?.type !== "CallExpression") return null;
  let callee = unwrap(current.callee);
  let args = current.arguments;
  if (callee?.type === "MemberExpression") {
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
  EXPECT_MODULES,
  expectArguments,
  isExpectCall,
  isExpectCallee,
  isNamespace,
  memberName,
  propertyName,
  unwrap,
};
