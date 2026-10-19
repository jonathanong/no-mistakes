"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { isTransparentExpression, traverse } = require("./async-ast");
const { contractFor } = require("./postgres-no-async-sql-builder-calls");
const { collectTypes } = require("./postgres-no-async-sql-builder-types");
const {
  configuredBuilders,
  isConfiguredBuilder,
} = require("./postgres-no-async-sql-builder-match");
const {
  bodyHasUnjudgedEffect,
  returnsJudgedSql,
} = require("./postgres-no-async-sql-builder-returns");

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
        "report async SQL builders that never wait only when every return is judged SQL and the contract allows a synchronous rewrite",
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
      if (
        !returnsJudgedSql(context, node, builders) ||
        bodyHasUnjudgedEffect(context, node, builders)
      )
        return;
      if (contractFor(node, context, types) !== "allows") return;
      context.report({ node, messageId: "removeAsync" });
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
