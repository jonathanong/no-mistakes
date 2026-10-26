"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const helpers = require("./postgres-runtime-helpers");
const { resolveVariable, unwrapTs } = require("./postgres-query-text");

const {
  executedQueryText,
  executorBindings,
  executorOptionDefaults,
  executorOptionSchema,
  firstCallArgument,
  isDatabaseCall,
  isManualTransactionText,
  isOwnerFile,
  sqlStatementBindings,
} = helpers;

// Only supplement legacy text recovery. Each hop stays in its declaring scope
// and reads a completed, unwritten const initializer; no runtime SQL evaluation.
function constChainText(argument, context) {
  let node = unwrapTs(argument);
  const seen = new Set();
  while (node?.type === "Identifier") {
    const variable = resolveVariable(node, context);
    if (!variable || seen.has(variable) || variable.defs.length !== 1) return null;
    seen.add(variable);
    const def = variable.defs[0];
    if (
      def.node.type !== "VariableDeclarator" ||
      def.node.id.type !== "Identifier" ||
      def.parent.kind !== "const" ||
      !def.node.init ||
      context.sourceCode.getScope(node) !== variable.scope ||
      def.node.init.range[1] > node.range[0] ||
      variable.references.some(
        (reference) =>
          (reference.isWrite() && !reference.init) ||
          context.sourceCode.getScope(reference.identifier) !== variable.scope,
      )
    )
      return null;
    node = unwrapTs(def.node.init);
  }
  if (node?.type === "Literal" && typeof node.value === "string") return node.value;
  if (node?.type === "TemplateLiteral" && node.expressions.length === 0)
    return node.quasis[0].value.cooked;
  return null;
}

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: {
        description:
          "disallow manual BEGIN, START TRANSACTION, COMMIT, and ROLLBACK executor calls",
        recommended: false,
      },
      schema: [
        executorOptionSchema({
          owners: { type: "array", items: { type: "string" } },
        }),
      ],
      messages: {
        manualTransaction:
          "Do not execute BEGIN, START TRANSACTION, COMMIT, or ROLLBACK through a query executor. Use withTransaction / withTransactionOptions so the owner helper owns transaction lifecycle.",
      },
    },
    (context) => {
      const options = executorOptionDefaults(context.options?.[0] ?? {});
      const suppressed = ruleSuppression(context, "postgres-no-manual-transaction");
      if (isOwnerFile(context.filename, options.owners)) return {};
      let bindings = new Set();
      let statements = new Map();

      return {
        Program(node) {
          // Transaction control must not disappear when a selected query receiver is unknown.
          bindings = executorBindings(node, options, context.filename, context, true);
          statements = sqlStatementBindings(node);
        },
        CallExpression(node) {
          if (suppressed(node)) return;
          if (!isDatabaseCall(node, bindings)) return;
          const argument = firstCallArgument(node, bindings);
          const text =
            executedQueryText(argument, statements, context) ?? constChainText(argument, context);
          if (!isManualTransactionText(text)) return;
          context.report({ node, messageId: "manualTransaction" });
        },
      };
    },
  ),
  { __test: helpers },
);
