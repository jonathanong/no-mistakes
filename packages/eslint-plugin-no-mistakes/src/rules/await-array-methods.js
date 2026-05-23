"use strict";

const { callMethodName, rule } = require("../helpers");

const BANNED_METHODS = new Set(["sort", "toSorted", "every", "findIndex", "slice", "toSpliced"]);

function unwrapChain(node) {
  return node?.type === "ChainExpression" ? node.expression : node;
}

function findVariable(node, context) {
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === node.name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function isReassignedBeforeUse(variable, node) {
  const definitionNames = new Set(variable.defs.map((def) => def.name));
  return variable.references.some(
    (reference) =>
      reference.isWrite() &&
      !definitionNames.has(reference.identifier) &&
      reference.identifier.range[0] < node.range[0],
  );
}

function isKnownArrayReceiver(node, context) {
  node = unwrapChain(node);
  if (node.type === "ArrayExpression") return true;
  if (node.type !== "Identifier") return false;
  const variable = findVariable(node, context);
  return Boolean(
    variable &&
    variable.defs.some(
      (def) =>
        def.type === "Variable" &&
        def.node?.id?.type === "Identifier" &&
        def.node.init?.type === "ArrayExpression",
    ) &&
    !isReassignedBeforeUse(variable, node),
  );
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
        const argument = unwrapChain(node.argument);
        if (argument.type !== "CallExpression") return;
        const method = callMethodName(argument);
        if (!BANNED_METHODS.has(method)) return;
        if (
          argument.callee.type !== "MemberExpression" ||
          !isKnownArrayReceiver(argument.callee.object, context)
        ) {
          return;
        }
        context.report({ node, messageId: "awaited", data: { method } });
      },
    };
  },
);
