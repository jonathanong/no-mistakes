"use strict";

const { isPromiseAllCall, isPromiseMethodCall } = require("./async-disposition-bindings");
const { callbackReturnsNormally } = require("./async-disposition-flow");
const { returnsThroughMock } = require("./async-disposition-mocks");
const {
  findContainingFunction,
  isFunction,
  isTransparentExpression,
  unwrapTransparentParent,
} = require("./async-ast");
const { memberPropertyName } = require("./async-targets");

function directlyDisposed(node, context, consumers) {
  const expression = unwrapTransparentParent(node);
  const parent = expression.parent;
  if (parent?.type === "ReturnStatement") {
    const fn = findContainingFunction(parent);
    return returnedFromFunction(fn, context, consumers);
  }
  if (parent?.type === "ArrowFunctionExpression" && parent.body === expression) {
    if (isCallCallee(parent)) return directlyDisposed(parent.parent, context, consumers);
    return returnedFromFunction(parent, context, consumers);
  }
  return (
    parent?.type === "AwaitExpression" ||
    (parent?.type === "UnaryExpression" && parent.operator === "void")
  );
}

function returnedFromFunction(fn, context, consumers) {
  if (!fn) return true;
  if (isCallArgument(fn))
    return returnsThroughMock(fn, context) || consumers.returnsThroughConsumer(fn);
  const named = consumers.namedDisposition(fn);
  return named ?? true;
}

function isCallArgument(node) {
  node = node && unwrapTransparentParent(node);
  return Boolean(node?.parent?.type === "CallExpression" && node.parent.arguments.includes(node));
}

function isCallCallee(node) {
  return node?.parent?.type === "CallExpression" && node.parent.callee === node;
}

function isPromiseAllIterable(node, promiseAll) {
  return promiseAll.arguments[0] === node && (node.type === "ArrayExpression" || isMapCall(node));
}

function isMapCall(node) {
  return (
    node?.type === "CallExpression" &&
    node.callee.type === "MemberExpression" &&
    memberPropertyName(node.callee) === "map"
  );
}

function isMapCallback(fn) {
  const call = fn.parent;
  return isMapCall(call) && call.arguments.includes(fn);
}

function expressionReturnedFromCallback(node, fn) {
  let current = node;
  let returnStatement = null;
  while (current && current !== fn) {
    if (current.parent === fn && fn.body === current && fn.body.type !== "BlockStatement") {
      return true;
    }
    if (current.parent?.type === "ReturnStatement") {
      returnStatement = current.parent;
    }
    current = current.parent;
  }
  return Boolean(returnStatement);
}

function canReachPromiseAll(node, promiseAll) {
  let current = node;
  // The caller supplies an ancestor aggregate; every path reaches it or rejects.
  for (;;) {
    const parent = current.parent;
    if (parent?.type === "MemberExpression") return false;
    if (parent === promiseAll) return isPromiseAllIterable(current, promiseAll);
    if (isFunction(parent)) {
      if (!expressionReturnedFromCallback(node, parent) || !isMapCallback(parent)) {
        return false;
      }
      current = parent.parent;
      continue;
    }
    if (parent?.type === "ArrayExpression") {
      if (parent.parent !== promiseAll || promiseAll.arguments[0] !== parent) return false;
      current = parent;
      continue;
    }
    if (parent?.type === "ObjectExpression") return false;
    if (
      parent?.type === "CallExpression" &&
      parent !== promiseAll &&
      parent.arguments.includes(current)
    ) {
      return false;
    }
    current = parent;
  }
}

function observedExpression(node, context) {
  let current = node;
  while (current.parent) {
    const parent = current.parent;
    if (isTransparentExpression(parent)) {
      current = parent;
      continue;
    }
    if (
      parent.type === "MemberExpression" &&
      parent.object === current &&
      parent.parent?.type === "CallExpression" &&
      parent.parent.callee === parent
    ) {
      current = parent;
      continue;
    }
    if (parent.type === "CallExpression" && parent.callee === current) {
      current = parent;
      continue;
    }
    if (isPromiseMethodCall(parent, context, ["resolve"]) && parent.arguments[0] === current) {
      current = parent;
      continue;
    }
    break;
  }
  return current;
}

function promiseAllDisposed(node, context, consumers) {
  let current = node.parent;
  while (current) {
    if (isPromiseAllCall(current, context) && canReachPromiseAll(node, current)) {
      return (
        directlyDisposed(current, context, consumers) ||
        directlyDisposed(observedExpression(current, context), context, consumers) ||
        promiseAllDisposed(current, context, consumers)
      );
    }
    current = current.parent;
  }
  return false;
}

function mappedPromise(node, disposed) {
  const parent = node.parent;
  const fn =
    parent.type === "ReturnStatement"
      ? findContainingFunction(parent)
      : isFunction(parent) && parent.body === node
        ? parent
        : null;
  // A synchronous mapper throw can lose promises from earlier iterations before
  // its result array exists. Async mapper throws become retained member rejections.
  return Boolean(
    fn &&
    isMapCallback(fn) &&
    (fn.async || fn.body.type !== "BlockStatement" || callbackReturnsNormally(fn.body)) &&
    disposed(fn.parent),
  );
}

module.exports = { directlyDisposed, observedExpression, promiseAllDisposed, mappedPromise };
