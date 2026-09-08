"use strict";

const { rule } = require("../helpers");
const helpers = require("./no-inline-noop-promise-catch-helpers");

const { isAllowedCallee, isInlineNoopFunction, rejectionHandler, shouldCheckFile } = helpers;

const patternList = { type: "array", items: { type: "string" } };

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: {
        description: "disallow inline Promise catch callbacks that do not handle the rejection",
        recommended: false,
      },
      schema: [
        {
          type: "object",
          properties: {
            checkedPathPatterns: patternList,
            allowedPathPatterns: patternList,
            allowedCalleeNamePatterns: patternList,
          },
          additionalProperties: false,
        },
      ],
      messages: {
        noopCatch:
          "Replace this inline no-op Promise catch with a named handler, logging, reporting, transformation, or rethrow so rejection handling stays reviewable.",
      },
    },
    (context) => {
      const options = context.options?.[0] ?? {};
      if (!shouldCheckFile(context.filename, options)) return {};
      return {
        CallExpression(node) {
          const handler = rejectionHandler(node);
          if (!handler || !isInlineNoopFunction(handler, context.sourceCode)) return;
          if (isAllowedCallee(node, options)) return;
          context.report({ node: handler, messageId: "noopCatch" });
        },
      };
    },
  ),
  { __test: helpers },
);
