"use strict";

const { unwrapSequence } = require("./async-ast");
const { staticImportSource } = require("./async-target-bindings");
const { requireSource } = require("./async-target-sources");
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
  member = unwrap(member);
  if (member?.type !== "MemberExpression") return null;
  const property = unwrap(member.property);
  if (member.computed && property?.type === "Identifier") return null;
  return propertyName(property);
}

function unwrap(node) {
  return unwrapSequence(node);
}

function allowedModule(source, modules) {
  return Boolean(source) && (modules || MESSAGE_MODULES).has(source);
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
  node = unwrap(node);
  if (allowedModule(requireSource(node, context), modules)) return true;
  if (node?.type !== "Identifier" || seen.has(node.name)) return false;
  const variable = resolveVariable(node, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (def?.type === "ImportBinding") {
    const source = staticImportSource(def);
    if (!allowedModule(source, modules)) return false;
    if (
      def.node.type === "ImportNamespaceSpecifier" ||
      def.node.type === "TSImportEqualsDeclaration"
    ) {
      return true;
    }
    if (def.node.type !== "ImportDefaultSpecifier") return false;
    return !modules
      ? source === "@playwright/test" || EXPECT_MODULES.has(source)
      : EXPECT_MODULES.has(source);
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
  if (allowedModule(requireSource(def.node.init, context), modules)) return true;
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
    requireExpectSource(def.node.init, context, modules);
  if (!fromModule) return false;
  return def.node.id.properties.some((property) => {
    let value = property.value;
    while (value?.type === "AssignmentPattern") value = value.left;
    const key = property.computed ? literalString(property.key) : propertyName(property.key);
    return value?.name === callee.name && key === "expect";
  });
}

function requireExpectSource(node, context, modules = null) {
  return allowedModule(requireSource(node, context), modules || MESSAGE_MODULES);
}

function spreadArguments(args) {
  const expanded = [];
  for (const arg of args || []) {
    if (arg?.type !== "SpreadElement") {
      expanded.push(arg);
      continue;
    }
    const list = unwrap(arg.argument);
    if (list?.type !== "ArrayExpression") return null;
    for (const element of list.elements) {
      if (!element || element.type === "SpreadElement") return null;
      expanded.push(element);
    }
  }
  return expanded;
}

function expectArguments(node, context) {
  const current = unwrap(node);
  if (current?.type !== "CallExpression") return null;
  let callee = unwrap(current.callee);
  let args = current.arguments;
  if (callee?.type === "MemberExpression") {
    const name = memberName(callee);
    if (name === "call") {
      const expanded = spreadArguments(args);
      if (!expanded) return null;
      callee = callee.object;
      args = expanded.slice(1);
    } else if (name === "apply") {
      const expanded = spreadArguments(args);
      if (!expanded) return null;
      const list = unwrap(expanded[1]);
      if (list?.type !== "ArrayExpression") return null;
      callee = callee.object;
      args = list.elements;
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
