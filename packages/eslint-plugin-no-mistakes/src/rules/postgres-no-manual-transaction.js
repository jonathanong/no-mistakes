"use strict";

const { rule } = require("../helpers");
const helpers = require("./postgres-runtime-helpers");

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

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: {
        description: "disallow manual BEGIN, COMMIT, and ROLLBACK executor calls",
        recommended: false,
      },
      schema: [
        executorOptionSchema({
          owners: { type: "array", items: { type: "string" } },
        }),
      ],
      messages: {
        manualTransaction:
          "Do not execute BEGIN, COMMIT, or ROLLBACK through a query executor. Use withTransaction / withTransactionOptions so the owner helper owns transaction lifecycle.",
      },
    },
    (context) => {
      const options = executorOptionDefaults(context.options?.[0] ?? {});
      if (isOwnerFile(context.filename, options.owners)) return {};
      let bindings = new Set();
      let statements = new Map();

      return {
        Program(node) {
          bindings = executorBindings(node, options);
          statements = sqlStatementBindings(node);
        },
        CallExpression(node) {
          if (!isDatabaseCall(node, bindings)) return;
          const argument = firstCallArgument(node);
          const text = executedQueryText(argument, statements, context);
          if (!isManualTransactionText(text)) return;
          context.report({ node, messageId: "manualTransaction" });
        },
      };
    },
  ),
  { __test: helpers },
);
