"use strict";

const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const {
  resolveVariable,
  stableConstInitializer,
  isReassigned,
} = require("./async-target-bindings");
const { mockInitializer } = require("./async-disposition-mock-bindings");
const { memberPropertyName } = require("./async-targets");

// Destructuring and loop targets also replace a consumer method's identity.
function propertyTargetWritten(value) {
  if (value.type !== "MemberExpression") return false;
  let target = value;
  while (
    ["ObjectPattern", "ArrayPattern", "RestElement"].includes(target.parent.type) ||
    (target.parent.type === "Property" && target.parent.value === target) ||
    (target.parent.type === "AssignmentPattern" && target.parent.left === target)
  )
    target = target.parent;
  const parent = target.parent;
  return (
    (["AssignmentExpression", "ForOfStatement", "ForInStatement"].includes(parent.type) &&
      parent.left === target) ||
    parent.type === "UpdateExpression" ||
    (parent.type === "UnaryExpression" && parent.operator === "delete")
  );
}

// Property writes invalidate provenance, including writes through stable local aliases.
// This follows syntax in this file, not arbitrary callees' side effects.
function propertiesWritten(
  node,
  context,
  seen = new Set(),
  initialization = null,
  propertiesOnly = false,
) {
  const variable = resolveVariable(node, context);
  const key = variable || node.name;
  if (seen.has(key)) return false;
  seen.add(key);
  const references = variable
    ? variable.references
    : context.sourceCode.scopeManager.globalScope.through.filter(
        (ref) => ref.identifier.name === node.name,
      );
  return references.some((reference) => {
    let value = unwrapTransparentParent(reference.identifier);
    for (;;) {
      const parent = value.parent;
      const member = parent.type === "MemberExpression" && parent.object === value;
      // These calls can return the existing spy; another call does not restore its method.
      const spyResult =
        parent.type === "CallExpression" &&
        parent.callee === value &&
        value.type === "MemberExpression" &&
        ["spyOn", "mocked", "mockImplementation", "mockImplementationOnce"].includes(
          memberPropertyName(value),
        );
      if (!member && !spyResult) break;
      value = unwrapTransparentParent(parent);
    }
    const parent = value.parent;
    if (parent === initialization && parent.left === value) return false;
    if (
      propertiesOnly
        ? propertyTargetWritten(value)
        : (parent.type === "AssignmentExpression" && parent.left === value) ||
          parent.type === "UpdateExpression" ||
          (parent.type === "UnaryExpression" && parent.operator === "delete")
    )
      return true;
    if (
      parent.type === "VariableDeclarator" &&
      parent.init === value &&
      parent.id.type === "Identifier" &&
      ["const", "let"].includes(parent.parent.kind)
    )
      return propertiesWritten(parent.id, context, seen, initialization, propertiesOnly);
    if (
      parent.type === "AssignmentExpression" &&
      parent.right === value &&
      parent.left.type === "Identifier"
    )
      return propertiesWritten(parent.left, context, seen, parent, propertiesOnly);
    return false;
  });
}

// Distinguish framework namespaces from the vi/jest object: vitest.fn is not vi.fn.
function frameworkObject(node, context, seen = new Set()) {
  node = unwrapExpression(node);
  if (node?.type === "MemberExpression") {
    const namespace = frameworkObject(node.object, context, seen);
    return (namespace === "vitest" && memberPropertyName(node) === "vi") ||
      (namespace === "@jest/globals" && memberPropertyName(node) === "jest")
      ? "api"
      : null;
  }
  if (
    node?.type !== "Identifier" ||
    isReassigned(node, context) ||
    propertiesWritten(node, context)
  )
    return null;
  const variable = resolveVariable(node, context);
  if (!variable?.defs.length) return node.name === "vi" || node.name === "jest" ? "api" : null;
  const def = variable.defs[0];
  if (def.type === "ImportBinding") {
    if (def.parent.importKind === "type" || def.node.importKind === "type") return null;
    const source = def.parent.source.value;
    if (source !== "vitest" && source !== "@jest/globals") return null;
    if (def.node.type === "ImportNamespaceSpecifier") return source;
    const name = def.node.imported?.name;
    return (source === "vitest" && name === "vi") || (source === "@jest/globals" && name === "jest")
      ? "api"
      : null;
  }
  const init = stableConstInitializer(node, context, seen);
  return init ? frameworkObject(init, context, seen) : null;
}

// mocked is an identity wrapper: visible writes to its input remain writes to
// the implementation method, even when the wrapper's result is never bound.
function mockedInputWritten(node, context) {
  node = unwrapExpression(node);
  while (node?.type === "MemberExpression") node = unwrapExpression(node.object);
  return node?.type === "Identifier" && propertiesWritten(node, context);
}

function mockCall(node, context, seen = new Set()) {
  node = unwrapExpression(node);
  if (node?.type === "Identifier") {
    const binding = mockInitializer(node, context, seen);
    if (!binding || propertiesWritten(node, context, new Set(), binding.assignment)) return false;
    seen.add(resolveVariable(node, context));
    return binding.init ? mockCall(binding.init, context, seen) : false;
  }
  if (node?.type !== "CallExpression") return false;
  const callee = unwrapExpression(node.callee);
  if (callee?.type !== "MemberExpression") return false;
  const method = memberPropertyName(callee);
  if (frameworkObject(callee.object, context) === "api") {
    if (method === "mocked") return !mockedInputWritten(node.arguments[0], context);
    return method === "fn" || method === "spyOn";
  }
  return (
    (method === "mockImplementation" || method === "mockImplementationOnce") &&
    mockCall(callee.object, context, seen)
  );
}

function returnsThroughMock(fn, context) {
  const callback = unwrapTransparentParent(fn);
  const call = callback.parent;
  if (call?.type !== "CallExpression" || call.arguments[0] !== callback) return false;
  const callee = unwrapExpression(call.callee);
  if (callee.type !== "MemberExpression") return false;
  const method = memberPropertyName(callee);
  if (method === "fn") return frameworkObject(callee.object, context) === "api";
  return (
    (method === "mockImplementation" || method === "mockImplementationOnce") &&
    mockCall(callee.object, context)
  );
}

module.exports = { propertiesWritten, returnsThroughMock };
