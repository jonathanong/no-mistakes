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
  for (const [key, value] of Object.entries(node)) {
    if (SKIP_KEYS.has(key) || !value) continue;
    const values = Array.isArray(value) ? value : [value];
    for (const child of values) {
      if (child && typeof child === "object" && typeof child.type === "string") {
        visit(child, callback, seen);
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
