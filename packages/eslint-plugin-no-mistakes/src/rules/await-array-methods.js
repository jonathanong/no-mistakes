"use strict";

const { callMethodName, rule } = require("../helpers");

const BANNED_METHODS = new Set(["sort", "toSorted", "every", "findIndex", "slice", "toSpliced"]);

function arrayVariableNames(program) {
  const names = new Set();
  for (const statement of program.body) {
    if (statement.type !== "VariableDeclaration") continue;
    for (const declaration of statement.declarations) {
      if (declaration.id.type === "Identifier" && declaration.init?.type === "ArrayExpression") {
        names.add(declaration.id.name);
      }
    }
  }
  return names;
}

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
      Program(node) {
        arrays = arrayVariableNames(node);
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
