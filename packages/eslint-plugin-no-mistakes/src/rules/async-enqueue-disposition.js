"use strict";

const { rule } = require("../helpers");
const { createTargetMatcher, memberPropertyName } = require("./async-targets");

function isFunction(node) {
  return (
    node?.type === "ArrowFunctionExpression" ||
    node?.type === "FunctionDeclaration" ||
    node?.type === "FunctionExpression"
  );
}

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
  const parent = node.parent;
  if (parent?.type === "ReturnStatement") {
    const fn = findContainingFunction(parent);
    return !isCallArgument(fn);
  }
  if (parent?.type === "ArrowFunctionExpression" && parent.body === node) {
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

function expressionReturnedFromCallback(node, fn) {
  let current = node;
  while (current && current !== fn) {
    if (current.parent === fn && fn.body === current && fn.body.type !== "BlockStatement") {
      return true;
    }
    if (current.parent?.type === "ReturnStatement" && current.parent.parent === fn.body) {
      return true;
    }
    current = current.parent;
  }
  return false;
}

function canReachPromiseAll(node, promiseAll) {
  let current = node;
  while (current && current !== promiseAll) {
    if (isFunction(current) && !expressionReturnedFromCallback(node, current)) {
      return false;
    }
    current = current.parent;
  }
  return current === promiseAll;
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
  return directlyDisposed(node) || promiseAllDisposed(node);
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
