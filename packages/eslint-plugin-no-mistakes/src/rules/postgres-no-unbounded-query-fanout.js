"use strict";

const { rule } = require("../helpers");
const helpers = require("./postgres-runtime-helpers");

const {
  callbackContainsExecutor,
  executorBindings,
  executorOptionDefaults,
  executorOptionSchema,
  isPromiseAllCallee,
  isStaticallyBounded,
  mapCallArgument,
} = helpers;

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: {
        description: "disallow unbounded Promise.all map fan-out of query executors",
        recommended: false,
      },
      schema: [
        executorOptionSchema({
          chunkFunctionNames: { type: "array", items: { type: "string" } },
        }),
      ],
      messages: {
        unboundedFanout:
          "Do not fan out unbounded mapped executor calls through Promise.all(). Use a static array, a SCREAMING_CASE constant, or a configured chunk helper first.",
      },
    },
    (context) => {
      const options = executorOptionDefaults(context.options?.[0] ?? {});
      let bindings = new Set();

      return {
        Program(node) {
          bindings = executorBindings(node, options);
        },
        CallExpression(node) {
          if (!isPromiseAllCallee(node.callee)) return;
          const mapped = mapCallArgument(node);
          if (!mapped) return;
          if (isStaticallyBounded(mapped.source, options.chunkFunctionNames, context)) return;
          if (!callbackContainsExecutor(mapped.callback, bindings, context)) return;
          context.report({ node, messageId: "unboundedFanout" });
        },
      };
    },
  ),
  { __test: helpers },
);
