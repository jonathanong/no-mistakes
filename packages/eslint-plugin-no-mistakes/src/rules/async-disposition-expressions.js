"use strict";

const {
  isPromiseAllCall,
  isPromiseMethodCall,
  retainedCollection,
} = require("./async-disposition-bindings");
const { callbackReturnsNormally, isMapCall, isMapCallback } = require("./async-disposition-flow");
const { assertionExpression, handledCatchStatement } = require("./async-disposition-observers");
const { returnsThroughMock } = require("./async-disposition-mocks");
const {
  findContainingFunction,
  isFunction,
  isTransparentExpression,
  unwrapTransparentParent,
} = require("./async-ast");

function directlyDisposed(node, context, consumers, origin = node) {
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
    handledCatchStatement(expression, context, origin) ||
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

function canReachPromiseAll(node, promiseAll, collectionRole) {
  let current = node;
  // The caller supplies an ancestor aggregate; every path reaches it or rejects.
  for (;;) {
    const parent = current.parent;
    if (parent?.type === "MemberExpression") return false;
    if (parent === promiseAll) return isPromiseAllIterable(current, promiseAll);
    if (isFunction(parent) || parent?.type === "ReturnStatement") {
      const fn = isFunction(parent) ? parent : findContainingFunction(parent);
      if (collectionRole || !isMapCallback(fn) || (isFunction(parent) && parent.body !== current))
        return false;
      current = unwrapTransparentParent(fn).parent;
      collectionRole = true;
      continue;
    }
    if (parent?.type === "ArrayExpression") {
      if (collectionRole) return false;
      current = parent;
      collectionRole = true;
      continue;
    }
    if (parent?.type === "SpreadElement") {
      // Spreading consumes a collection; unspread collections stay nested.
      if (!collectionRole || parent.parent?.type !== "ArrayExpression") return false;
      current = parent;
      collectionRole = false;
      continue;
    }
    // Only wrappers that preserve this exact value may reach an aggregate.
    if (
      !isTransparentExpression(parent) &&
      !(
        parent?.type === "ConditionalExpression" &&
        (parent.consequent === current || parent.alternate === current)
      )
    )
      return false;
    current = parent;
  }
}

function observedExpression(node, context) {
  let current = assertionExpression(node, context) || node;
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

function promiseAllDisposed(node, context, consumers, collectionRole = false) {
  let current = node.parent;
  while (current) {
    if (isPromiseAllCall(current, context) && canReachPromiseAll(node, current, collectionRole)) {
      return (
        directlyDisposed(current, context, consumers) ||
        directlyDisposed(observedExpression(current, context), context, consumers, current) ||
        promiseAllDisposed(current, context, consumers)
      );
    }
    current = current.parent;
  }
  return false;
}

function mappedCollectionDisposed(call, context, consumers, disposed) {
  return (
    promiseAllDisposed(call, context, consumers, true) ||
    retainedCollection(call, context, (value) => isPromiseAllCall(value, context), disposed)
  );
}

function mapperCompletes(fn) {
  return fn.async || fn.body.type !== "BlockStatement" || callbackReturnsNormally(fn.body);
}

function mappedArgumentDisposed(fn, argument, context, consumers, disposed) {
  const value = unwrapTransparentParent(argument);
  const call = value.parent;
  return (
    isMapCall(call) &&
    call.arguments[0] === value &&
    mapperCompletes(fn) &&
    mappedCollectionDisposed(call, context, consumers, disposed)
  );
}

function mappedPromise(node, context, consumers, disposed) {
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
    mapperCompletes(fn) &&
    mappedCollectionDisposed(unwrapTransparentParent(fn).parent, context, consumers, disposed),
  );
}

module.exports = {
  directlyDisposed,
  observedExpression,
  promiseAllDisposed,
  mappedArgumentDisposed,
  mappedPromise,
};
