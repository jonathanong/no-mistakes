"use strict";

const {
  findContainingFunction,
  unwrapExpression,
  unwrapTransparentParent,
} = require("./async-ast");
const { resolveVariable, stableConstInitializer } = require("./async-target-bindings");
const { memberPropertyName } = require("./async-targets");

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

module.exports = { collectedPromise, isPromiseAllCall, retainedPromise };
