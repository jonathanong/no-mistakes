"use strict";

const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const {
  resolveVariable,
  stableConstInitializer,
  isReassigned,
} = require("./async-target-bindings");
const { memberPropertyName } = require("./async-targets");

// Property writes invalidate provenance, including writes through stable local aliases.
// This follows syntax in this file, not arbitrary callees' side effects.
function propertiesWritten(node, context, seen = new Set()) {
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
    while (value.parent?.type === "MemberExpression" && value.parent.object === value)
      value = unwrapTransparentParent(value.parent);
    const parent = value.parent;
    if (
      (parent.type === "AssignmentExpression" && parent.left === value) ||
      parent.type === "UpdateExpression" ||
      (parent.type === "UnaryExpression" && parent.operator === "delete")
    )
      return true;
    if (
      parent.type === "VariableDeclarator" &&
      parent.init === value &&
      parent.id.type === "Identifier" &&
      parent.parent.kind === "const"
    )
      return propertiesWritten(parent.id, context, seen);
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

function mockCall(node, context, seen = new Set()) {
  node = unwrapExpression(node);
  if (node?.type === "Identifier") {
    if (propertiesWritten(node, context)) return false;
    const init = stableConstInitializer(node, context, seen);
    return init ? mockCall(init, context, seen) : false;
  }
  if (node?.type !== "CallExpression") return false;
  const callee = unwrapExpression(node.callee);
  if (callee?.type !== "MemberExpression") return false;
  const method = memberPropertyName(callee);
  if (frameworkObject(callee.object, context) === "api")
    return method === "fn" || method === "spyOn";
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

module.exports = { returnsThroughMock };
