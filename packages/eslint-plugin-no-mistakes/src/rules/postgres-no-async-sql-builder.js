"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { isTransparentExpression, traverse } = require("./async-ast");
const { collectTypes, contractFor } = require("./postgres-no-async-sql-builder-contract");
const {
  configuredBuilders,
  isConfiguredBuilder,
} = require("./postgres-no-async-sql-builder-match");

function importSuspends(node, func) {
  let current = node;
  while (current && current !== func) {
    const parent = current.parent;
    if (!parent || parent === func) {
      return func.type === "ArrowFunctionExpression" && func.body === current;
    }
    if (parent.type === "ReturnStatement" && parent.argument === current) return true;
    // `return import()?.then(...)` is still a returned dynamic import. The
    // promise chain suspends; a bare statement-level import does not.
    if (
      isTransparentExpression(parent) ||
      parent.type === "ConditionalExpression" ||
      parent.type === "LogicalExpression" ||
      parent.type === "SequenceExpression" ||
      (parent.type === "MemberExpression" && parent.object === current) ||
      (parent.type === "CallExpression" && parent.callee === current)
    ) {
      current = parent;
      continue;
    }
    return false;
  }
  return false;
}

function functionSuspends(context, func) {
  let suspended = false;
  const visit = (node) => {
    if (suspended) return;
    if (node.type === "AwaitExpression") suspended = true;
    else if (node.type === "ForOfStatement" && node.await) suspended = true;
    else if (node.type === "VariableDeclaration" && node.kind === "await using") suspended = true;
    else if (node.type === "ImportExpression" && importSuspends(node, func)) suspended = true;
  };
  for (const parameter of func.params) traverse(context, parameter, visit);
  traverse(context, func.body, visit);
  return suspended;
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description:
        "report async SQL builders that never wait, and suggest removing async only when the contract allows it",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        additionalProperties: false,
        properties: {
          builders: {
            type: "array",
            items: {
              type: "object",
              additionalProperties: false,
              properties: {
                names: { type: "array", items: { type: "string", minLength: 1 } },
                callees: { type: "array", items: { type: "string", minLength: 1 } },
                source: { type: "string", minLength: 1 },
              },
            },
          },
        },
      },
    ],
    messages: {
      removeAsync:
        "This async SQL builder never waits. Remove async and return the same literal SQL synchronously so the analyzer can judge it; callers that only await the call can call it directly. No autofix is offered.",
      keepAsync:
        "This async SQL builder never waits, but the visible contract still requires a Promise. Do not remove async. No autofix is offered.",
    },
  },
  (context) => {
    const builders = configuredBuilders(context.options?.[0]);
    if (builders.length === 0) return {};
    const suppressed = ruleSuppression(context, "postgres-no-async-sql-builder");
    let types = null;

    function visit(node) {
      if (!node.async || node.generator || suppressed(node)) return;
      if (!isConfiguredBuilder(node, builders, context) || functionSuspends(context, node)) return;
      const decision = contractFor(node, context, types);
      if (decision === "unknown") return;
      context.report({
        node,
        messageId: decision === "allows" ? "removeAsync" : "keepAsync",
      });
    }

    return {
      Program(node) {
        types = collectTypes(context, node);
      },
      FunctionDeclaration: visit,
      FunctionExpression: visit,
      ArrowFunctionExpression: visit,
    };
  },
);
