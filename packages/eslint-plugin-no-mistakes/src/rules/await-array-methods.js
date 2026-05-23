"use strict";

const { callMethodName, rule } = require("../helpers");

const BANNED_METHODS = new Set(["sort", "toSorted", "every", "findIndex", "slice", "toSpliced"]);

function isKnownArrayReceiver(node, context) {
  if (node.type === "ArrayExpression") return true;
  if (node.type !== "Identifier") return false;
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === node.name);
    if (variable) {
      return variable.defs.some(
        (def) => def.type === "Variable" && def.node?.init?.type === "ArrayExpression",
      );
    }
    scope = scope.upper;
  }
  return false;
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
    return {
      AwaitExpression(node) {
        if (node.argument.type !== "CallExpression") return;
        const method = callMethodName(node.argument);
        if (!BANNED_METHODS.has(method)) return;
        if (
          node.argument.callee.type !== "MemberExpression" ||
          !isKnownArrayReceiver(node.argument.callee.object, context)
        ) {
          return;
        }
        context.report({ node, messageId: "awaited", data: { method } });
      },
    };
  },
);
