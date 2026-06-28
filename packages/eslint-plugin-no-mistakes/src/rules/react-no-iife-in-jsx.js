"use strict";

const { rule } = require("../helpers");

const SKIP_KEYS = new Set(["comments", "loc", "parent", "range", "tokens"]);

function isIife(node) {
  return (
    node?.type === "CallExpression" &&
    ["ArrowFunctionExpression", "FunctionExpression"].includes(node.callee?.type)
  );
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

module.exports = rule(
  {
    type: "suggestion",
    docs: { description: "disallow immediately invoked functions inside JSX", recommended: false },
    schema: [],
    messages: { iife: "Avoid IIFEs in JSX; extract the logic into a variable or component." },
  },
  (context) => ({
    JSXExpressionContainer(node) {
      visit(node.expression, (child) => {
        if (isIife(child)) context.report({ node: child, messageId: "iife" });
      });
    },
  }),
);
