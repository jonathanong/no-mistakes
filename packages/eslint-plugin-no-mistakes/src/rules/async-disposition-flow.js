"use strict";

const { findContainingFunction, unwrapTransparentParent } = require("./async-ast");
const { memberPropertyName } = require("./async-targets");

function statement(node) {
  while (node.parent && node.parent.type !== "BlockStatement" && node.parent.type !== "Program")
    node = node.parent;
  return node;
}

function unconditional(node, end) {
  while (node !== end) {
    if (
      ["LogicalExpression", "ConditionalExpression", "IfStatement"].includes(node.type) ||
      node.optional
    )
      return false;
    node = node.parent;
  }
  return true;
}

// A conditional push may precede an independently proven return of the promise.
// Abrupt exits, loops, and unknown statement kinds cannot establish that return.
function fallsThrough(node) {
  if (["ExpressionStatement", "VariableDeclaration", "EmptyStatement"].includes(node.type))
    return true;
  if (node.type === "BlockStatement") return node.body.every(fallsThrough);
  return (
    node.type === "IfStatement" &&
    fallsThrough(node.consequent) &&
    (!node.alternate || fallsThrough(node.alternate))
  );
}

function straightLine(from, to) {
  const start = statement(from);
  const end = statement(to);
  if (
    ["ReturnStatement", "ThrowStatement"].includes(start.type) ||
    start.parent !== end.parent ||
    start.range[0] >= end.range[0] ||
    !unconditional(to, end)
  )
    return false;
  if (!["ExpressionStatement", "ReturnStatement", "VariableDeclaration"].includes(end.type))
    return false;
  const body = start.parent.body;
  return body.slice(body.indexOf(start) + 1, body.indexOf(end)).every(fallsThrough);
}

function collectionAnchor(push) {
  const fn = findContainingFunction(push);
  const call = fn && unwrapTransparentParent(fn).parent;
  if (
    call?.type === "CallExpression" &&
    call.arguments[0] === unwrapTransparentParent(fn) &&
    call.callee.type === "MemberExpression" &&
    memberPropertyName(call.callee) === "forEach"
  ) {
    const invocation = statement(call);
    if (!unconditional(call, invocation) || invocation.type === "IfStatement") return null;
    if (fn.body === push) return fn.async ? null : call;
    const start = statement(push);
    if (
      fn.async ||
      fn.generator ||
      start.parent !== fn.body ||
      start.type !== "ExpressionStatement" ||
      !unconditional(push, start) ||
      !fn.body.body.slice(0, fn.body.body.indexOf(start)).every(fallsThrough)
    )
      return null;
    return call;
  }
  const start = statement(push);
  return unconditional(push, start) && start.type !== "IfStatement" ? push : null;
}

function laterInBlock(from, to) {
  if (findContainingFunction(from) !== findContainingFunction(to)) return false;
  return straightLine(from, to);
}

function callbackReturnsNormally(node) {
  if (node.type === "ReturnStatement") return true;
  if (node.type === "BlockStatement") return node.body.every(callbackReturnsNormally);
  if (node.type === "IfStatement")
    return (
      callbackReturnsNormally(node.consequent) &&
      (!node.alternate || callbackReturnsNormally(node.alternate))
    );
  return fallsThrough(node);
}

function callbackCompletes(push) {
  const fn = findContainingFunction(push);
  if (fn.body.type !== "BlockStatement") return true;
  const start = statement(push);
  // A callback return completes this iteration; a throw can abort the caller
  // before an ordinary outer drain, but still transfers control to finally.
  return fn.body.body.slice(fn.body.body.indexOf(start) + 1).every(callbackReturnsNormally);
}

function conditionalPushAnchor(push) {
  let start = statement(push);
  while (start.parent?.type === "BlockStatement" && start.parent.parent?.type === "IfStatement")
    start = start.parent.parent;
  return start.type === "IfStatement" &&
    findContainingFunction(start) === findContainingFunction(push) &&
    fallsThrough(start)
    ? start
    : null;
}

function laterDrain(from, to, createdHere = false) {
  // Only a value created inside the conditional push is necessarily retained.
  // A promise created beforehand may remain unhandled when the branch is skipped.
  const conditional = createdHere && conditionalPushAnchor(from);
  const anchor = conditional || collectionAnchor(from);
  if (!anchor || findContainingFunction(anchor) !== findContainingFunction(to)) return false;
  const completes = conditional || anchor === from || callbackCompletes(from);
  if (completes && straightLine(anchor, to)) return true;
  const end = statement(to);
  const finalizer = end.parent;
  const attempt = finalizer.parent;
  if (
    attempt?.type !== "TryStatement" ||
    attempt.finalizer !== finalizer ||
    !unconditional(to, end) ||
    !["ExpressionStatement", "ReturnStatement", "VariableDeclaration"].includes(end.type) ||
    !finalizer.body.slice(0, finalizer.body.indexOf(end)).every(fallsThrough)
  )
    return false;
  const start = statement(anchor);
  // Finally executes on both normal completion and return/throw from this try.
  if (start.parent === attempt.block)
    return unconditional(anchor, start) && (conditional || start.type !== "IfStatement");
  if (
    !completes ||
    ["ReturnStatement", "ThrowStatement"].includes(start.type) ||
    start.parent !== attempt.parent ||
    start.range[0] >= attempt.range[0]
  )
    return false;
  const body = start.parent.body;
  return body.slice(body.indexOf(start) + 1, body.indexOf(attempt)).every(fallsThrough);
}

module.exports = { callbackReturnsNormally, collectionAnchor, laterDrain, laterInBlock, statement };
