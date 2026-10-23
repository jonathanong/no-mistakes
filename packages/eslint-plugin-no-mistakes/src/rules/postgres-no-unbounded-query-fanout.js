"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const helpers = require("./postgres-runtime-helpers");
const { fanoutMapCall } = require("./postgres-fanout-calls");

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
      const suppressed = ruleSuppression(context, "postgres-no-unbounded-query-fanout");
      let bindings = new Set();

      return {
        Program(node) {
          bindings = executorBindings(node, options, context.filename, context);
        },
        CallExpression(node) {
          if (suppressed(node)) return;
          // Preserve direct-call recall while adding stricter static forwarding.
          const mapped =
            (isPromiseAllCallee(node.callee) && mapCallArgument(node)) ||
            fanoutMapCall(node, isPromiseAllCallee, context);
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
