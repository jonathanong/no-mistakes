"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const {
  collectedPromise,
  isPromiseAllCall,
  retainedPromise,
} = require("./async-disposition-bindings");
const { returnsThroughMock } = require("./async-disposition-mocks");
const {
  findContainingFunction,
  isFunction,
  isTransparentExpression,
  unwrapTransparentParent,
} = require("./async-ast");
const { targetOptionsSchema } = require("./async-schema");
const { createTargetMatcher, memberPropertyName } = require("./async-targets");

function directlyDisposed(node, context) {
  const expression = unwrapTransparentParent(node);
  const parent = expression.parent;
  if (parent?.type === "ReturnStatement") {
    const fn = findContainingFunction(parent);
    return !isCallArgument(fn) || returnsThroughMock(fn, context);
  }
  if (parent?.type === "ArrowFunctionExpression" && parent.body === expression) {
    if (isCallCallee(parent)) return directlyDisposed(parent.parent, context);
    return !isCallArgument(parent) || returnsThroughMock(parent, context);
  }
  return (
    parent?.type === "AwaitExpression" ||
    (parent?.type === "UnaryExpression" && parent.operator === "void")
  );
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
  while (current && current !== promiseAll) {
    const parent = current.parent;
    if (isTransparentExpression(parent)) {
      current = parent;
      continue;
    }
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
  return current === promiseAll;
}

function observedExpression(node) {
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
    break;
  }
  return current;
}

function promiseAllDisposed(node, context) {
  let current = node.parent;
  while (current) {
    if (isPromiseAllCall(current, context) && canReachPromiseAll(node, current)) {
      return (
        directlyDisposed(current, context) ||
        directlyDisposed(observedExpression(current), context) ||
        promiseAllDisposed(current, context)
      );
    }
    current = current.parent;
  }
  return false;
}

function isDisposed(node, context, seen = new Set()) {
  if (seen.has(node)) return false;
  seen.add(node);
  const observed = observedExpression(node);
  const disposed = (value) => isDisposed(value, context, seen);
  return (
    directlyDisposed(node, context) ||
    directlyDisposed(observed, context) ||
    promiseAllDisposed(node, context) ||
    retainedPromise(observed, context, disposed) ||
    collectedPromise(observed, context, (value) => isPromiseAllCall(value, context), disposed)
  );
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require explicit async disposition for configured async calls",
      recommended: false,
    },
    hasSuggestions: true,
    schema: targetOptionsSchema,
    messages: {
      disposition:
        "Handle this async promise explicitly with await, return, Promise.all(...), or void.",
      markVoid: "Prefix the call with void to mark this promise as intentionally floating.",
    },
  },
  (context) => {
    const matcher = createTargetMatcher(context);
    const suppressed = ruleSuppression(context, "async-call-disposition");
    if (!matcher.hasTargets) return {};

    return {
      ...matcher.visitors,
      CallExpression(node) {
        if (!matcher.isTargetCall(node) || suppressed(node) || isDisposed(node, context)) return;
        context.report({
          node,
          messageId: "disposition",
          suggest:
            node.parent?.type === "ExpressionStatement"
              ? [
                  {
                    messageId: "markVoid",
                    fix(fixer) {
                      return fixer.insertTextBefore(node, "void ");
                    },
                  },
                ]
              : [],
        });
      },
    };
  },
);
