"use strict";

const { callMethodName, rule } = require("../helpers");

const BANNED_METHODS = new Set(["sort", "toSorted", "every", "findIndex", "slice", "toSpliced"]);

function isKnownArrayReceiver(node, names) {
  if (node.type === "ArrayExpression") return true;
  return node.type === "Identifier" && names.has(node.name);
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow awaiting synchronous array methods",
      recommended: false,
    },
    schema: [],
    messages: {
      awaited:
        "Do not await {{method}}(). This array method returns a synchronous value; remove await or await the async work explicitly.",
    },
  },
  (context) => {
    let arrays = new Set();
    return {
      VariableDeclarator(node) {
        if (node.id.type === "Identifier" && node.init?.type === "ArrayExpression") {
          arrays.add(node.id.name);
        }
      },
      AwaitExpression(node) {
        if (node.argument.type !== "CallExpression") return;
        const method = callMethodName(node.argument);
        if (!BANNED_METHODS.has(method)) return;
        if (
          node.argument.callee.type !== "MemberExpression" ||
          !isKnownArrayReceiver(node.argument.callee.object, arrays)
        ) {
          return;
        }
        context.report({ node, messageId: "awaited", data: { method } });
      },
    };
  },
);
