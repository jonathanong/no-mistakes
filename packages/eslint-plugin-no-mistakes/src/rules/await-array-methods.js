"use strict";

const { literalString, rule } = require("../helpers");
const { unwrapExpression } = require("./async-ast");
const { logicalCall } = require("./module-mock-helpers");

const BANNED_METHODS = new Set(["sort", "toSorted", "every", "findIndex", "slice", "toSpliced"]);

function methodName(callee) {
  if (!callee.computed) return callee.property.type === "Identifier" ? callee.property.name : null;
  if (callee.property.type === "Literal") return String(callee.property.value);
  return literalString(callee.property);
}

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

function isKnownArrayReceiver(node, context, seen = new Set()) {
  node = unwrapExpression(node);
  if (!node) return false;
  if (node.type === "ArrayExpression") return true;
  if (node.type !== "Identifier") return false;
  const variable = findVariable(node, context);
  if (!variable || seen.has(variable) || isReassignedBeforeUse(variable, node)) return false;
  seen.add(variable);
  return variable.defs.some((def) => {
    if (def.type !== "Variable" || def.node?.id?.type !== "Identifier") return false;
    if (def.node.init?.type === "ArrayExpression") return true;
    return def.parent?.kind === "const" && isKnownArrayReceiver(def.node.init, context, seen);
  });
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
        const invoked = logicalCall(argument);
        if (invoked?.callee?.type !== "MemberExpression") return;
        const method = methodName(invoked.callee);
        if (!BANNED_METHODS.has(method)) return;
        if (!isKnownArrayReceiver(invoked.callee.object, context)) return;
        context.report({ node, messageId: "awaited", data: { method } });
      },
    };
  },
);
