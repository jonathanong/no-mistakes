"use strict";

const { rule } = require("../helpers");
const {
  createTargetMatcher,
  isFunction,
  isTransparentExpression,
  memberPropertyName,
  unwrapTransparentParent,
} = require("./async-targets");

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
    return !isCallArgument(parent);
  }
  return (
    parent?.type === "AwaitExpression" ||
    (parent?.type === "UnaryExpression" && parent.operator === "void")
  );
}

function findContainingFunction(node) {
  let current = node.parent;
  while (current) {
    if (isFunction(current)) return current;
    current = current.parent;
  }
}

function isCallArgument(node) {
  return Boolean(node?.parent?.type === "CallExpression" && node.parent.arguments.includes(node));
}

function isMapCallback(fn) {
  const call = fn.parent;
  return (
    call?.type === "CallExpression" &&
    call.arguments.includes(fn) &&
    call.callee.type === "MemberExpression" &&
    memberPropertyName(call.callee) === "map"
  );
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
    if (parent.type === "MemberExpression" && parent.object === current) {
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
      return directlyDisposed(current) || promiseAllDisposed(current);
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
    schema: [
      {
        type: "object",
        properties: {
          targets: {
            type: "array",
            items: {
              type: "object",
              properties: {
                sourcePatterns: { type: "array", items: { type: "string" } },
                calleeNamePatterns: { type: "array", items: { type: "string" } },
              },
              additionalProperties: false,
            },
          },
        },
        additionalProperties: false,
      },
    ],
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
