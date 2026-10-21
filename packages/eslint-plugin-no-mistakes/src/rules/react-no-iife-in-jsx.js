"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { literalString } = require("./module-mock-helpers");

const SKIP_KEYS = new Set(["comments", "loc", "parent", "range", "tokens"]);

function unwrap(node) {
  while (node?.type === "ChainExpression") node = node.expression;
  if (node?.type === "SequenceExpression" && node.expressions?.length) {
    return unwrap(node.expressions.at(-1));
  }
  return node;
}

function isFunctionCallee(node) {
  const current = unwrap(node);
  if (["ArrowFunctionExpression", "FunctionExpression"].includes(current?.type)) return true;
  if (current?.type !== "MemberExpression" || current.computed) return false;
  const name =
    current.property?.type === "Identifier"
      ? current.property.name
      : literalString(current.property);
  return (name === "call" || name === "apply") && isFunctionCallee(current.object);
}

function isIife(node) {
  return node?.type === "CallExpression" && isFunctionCallee(node.callee);
}

function visit(node, callback, seen = new Set()) {
  if (!node || seen.has(node)) return;
  seen.add(node);
  callback(node);
  for (const key in node) {
    if (!Object.hasOwn(node, key) || SKIP_KEYS.has(key)) continue;
    const value = node[key];
    if (!value) continue;
    if (Array.isArray(value)) {
      for (let i = 0; i < value.length; i++) {
        const child = value[i];
        if (child && typeof child === "object" && typeof child.type === "string") {
          visit(child, callback, seen);
        }
      }
    } else {
      if (typeof value === "object" && typeof value.type === "string") {
        visit(value, callback, seen);
      }
    }
  }
}

module.exports = Object.assign(rule(
  {
    type: "suggestion",
    docs: { description: "disallow immediately invoked functions inside JSX", recommended: false },
    schema: [],
    messages: { iife: "Avoid IIFEs in JSX; extract the logic into a variable or component." },
  },
  (context) => {
    const suppressed = ruleSuppression(context, "react-no-iife-in-jsx");
    return {
      JSXExpressionContainer(node) {
        visit(node.expression, (child) => {
          if (isIife(child) && !suppressed(child))
            context.report({ node: child, messageId: "iife" });
        });
      },
    };
  },
),
{ __test: { isFunctionCallee, isIife } },
);
