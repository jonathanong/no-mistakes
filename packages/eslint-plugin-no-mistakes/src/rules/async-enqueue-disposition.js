"use strict";

const { rule } = require("../helpers");
const {
  findContainingFunction,
  isFunction,
  isTransparentExpression,
  unwrapTransparentParent,
} = require("./async-ast");
const { targetOptionsSchema } = require("./async-schema");
const { createTargetMatcher, memberPropertyName } = require("./async-targets");

function isPromiseAllCall(node) {
  return (
    node?.type === "CallExpression" &&
    node.callee.type === "MemberExpression" &&
    !node.callee.computed &&
    node.callee.object.type === "Identifier" &&
    node.callee.object.name === "Promise" &&
    memberPropertyName(node.callee) === "all"
  );
}

function directlyDisposed(node) {
  const expression = unwrapTransparentParent(node);
  const parent = expression.parent;
  if (parent?.type === "ReturnStatement") {
    const fn = findContainingFunction(parent);
    return !isCallArgument(fn);
  }
  if (parent?.type === "ArrowFunctionExpression" && parent.body === expression) {
    if (isCallCallee(parent)) return directlyDisposed(parent.parent);
    return !isCallArgument(parent);
  }
  return (
    parent?.type === "AwaitExpression" ||
    (parent?.type === "UnaryExpression" && parent.operator === "void")
  );
}

function isCallArgument(node) {
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

function promiseAllDisposed(node) {
  let current = node.parent;
  while (current) {
    if (isPromiseAllCall(current) && canReachPromiseAll(node, current)) {
      return (
        directlyDisposed(current) ||
        directlyDisposed(observedExpression(current)) ||
        promiseAllDisposed(current)
      );
    }
    current = current.parent;
  }
  return false;
}

function isDisposed(node) {
  const observed = observedExpression(node);
  return directlyDisposed(node) || directlyDisposed(observed) || promiseAllDisposed(node);
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require explicit async disposition for configured enqueue calls",
      recommended: false,
    },
    fixable: "code",
    schema: targetOptionsSchema,
    messages: {
      disposition:
        "Handle this enqueue promise explicitly with await, return, Promise.all(...), or void.",
    },
  },
  (context) => {
    const matcher = createTargetMatcher(context);
    if (!matcher.hasTargets) return {};

    return {
      ...matcher.visitors,
      CallExpression(node) {
        if (!matcher.isTargetCall(node) || isDisposed(node)) return;
        context.report({
          node,
          messageId: "disposition",
          fix(fixer) {
            if (node.parent?.type !== "ExpressionStatement") return null;
            return fixer.insertTextBefore(node, "void ");
          },
        });
      },
    };
  },
);
