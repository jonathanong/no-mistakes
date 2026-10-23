"use strict";

const {
  findContainingFunction,
  unwrapExpression,
  unwrapTransparentParent,
} = require("./async-ast");
const {
  resolveVariable,
  stableConstInitializer,
  isReassigned,
} = require("./async-target-bindings");
const { memberPropertyName } = require("./async-targets");

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
  if (node?.type !== "Identifier" || isReassigned(node, context)) return null;
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

function globalPromise(node, context) {
  const variable = resolveVariable(node, context);
  return !variable?.defs.length;
}

function isPromiseAllCall(node, context) {
  const callee = unwrapExpression(node?.callee);
  const object = unwrapExpression(callee?.object);
  return (
    node?.type === "CallExpression" &&
    callee?.type === "MemberExpression" &&
    object?.type === "Identifier" &&
    object.name === "Promise" &&
    globalPromise(object, context) &&
    ["all", "allSettled"].includes(memberPropertyName(callee))
  );
}

function statement(node) {
  while (node.parent && node.parent.type !== "BlockStatement" && node.parent.type !== "Program")
    node = node.parent;
  return node;
}

function unconditional(node, end) {
  while (node !== end) {
    if (node.type === "LogicalExpression" || node.type === "ConditionalExpression" || node.optional)
      return false;
    node = node.parent;
  }
  return true;
}

// Only straight-line statements establish that the later observation executes.
function laterInBlock(from, to) {
  const start = statement(from);
  const end = statement(to);
  if (
    findContainingFunction(from) !== findContainingFunction(to) ||
    start.parent !== end.parent ||
    start.range[0] >= end.range[0] ||
    !["ExpressionStatement", "ReturnStatement", "VariableDeclaration"].includes(end.type)
  )
    return false;
  if (!unconditional(to, end)) return false;
  const body = start.parent.body;
  return body
    .slice(body.indexOf(start) + 1, body.indexOf(end))
    .every((node) => node.type === "ExpressionStatement" || node.type === "VariableDeclaration");
}

function pushedArray(node) {
  const value = unwrapTransparentParent(node);
  const push = value.parent;
  const callee = unwrapExpression(push?.callee);
  if (
    push?.type !== "CallExpression" ||
    !push.arguments.includes(value) ||
    callee?.type !== "MemberExpression" ||
    memberPropertyName(callee) !== "push" ||
    callee.object.type !== "Identifier" ||
    push.parent.type !== "ExpressionStatement"
  )
    return null;
  return { id: callee.object, push };
}

function collectedPromise(node, context, isAggregate, disposed) {
  const found = pushedArray(node);
  if (!found) return false;
  const init = unwrapExpression(stableConstInitializer(found.id, context, new Set()));
  if (init?.type !== "ArrayExpression" || init.elements.length !== 0) return false;
  const variable = resolveVariable(found.id, context);
  let drained = false;
  for (const reference of variable.references) {
    if (reference.init) continue;
    const id = reference.identifier;
    const parent = unwrapTransparentParent(id).parent;
    if (isAggregate(parent) && parent.arguments[0] === id) {
      if (laterInBlock(found.push, parent) && disposed(parent)) drained = true;
      continue;
    }
    const member = parent;
    const call = member?.parent;
    if (
      member?.type !== "MemberExpression" ||
      member.object !== id ||
      memberPropertyName(member) !== "push" ||
      call?.type !== "CallExpression" ||
      call.callee !== member ||
      call.parent.type !== "ExpressionStatement" ||
      statement(call).parent !== statement(found.push).parent
    )
      return false;
  }
  return drained;
}

function retainedPromise(node, context, disposed) {
  const expression = unwrapTransparentParent(node);
  const declaration = expression.parent;
  if (
    declaration?.type !== "VariableDeclarator" ||
    declaration.init !== expression ||
    declaration.id.type !== "Identifier"
  )
    return false;
  if (!stableConstInitializer(declaration.id, context, new Set())) return false;
  const variable = resolveVariable(declaration.id, context);
  return variable.references.some(
    (reference) =>
      !reference.init && laterInBlock(node, reference.identifier) && disposed(reference.identifier),
  );
}

module.exports = { collectedPromise, isPromiseAllCall, retainedPromise, returnsThroughMock };
