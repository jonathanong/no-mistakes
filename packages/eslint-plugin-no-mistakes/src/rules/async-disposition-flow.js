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

function laterDrain(from, to) {
  const anchor = collectionAnchor(from);
  if (!anchor || findContainingFunction(anchor) !== findContainingFunction(to)) return false;
  if (straightLine(anchor, to)) return true;
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
    return unconditional(anchor, start) && start.type !== "IfStatement";
  if (
    ["ReturnStatement", "ThrowStatement"].includes(start.type) ||
    start.parent !== attempt.parent ||
    start.range[0] >= attempt.range[0]
  )
    return false;
  const body = start.parent.body;
  return body.slice(body.indexOf(start) + 1, body.indexOf(attempt)).every(fallsThrough);
}

module.exports = { collectionAnchor, laterDrain, laterInBlock, statement };
