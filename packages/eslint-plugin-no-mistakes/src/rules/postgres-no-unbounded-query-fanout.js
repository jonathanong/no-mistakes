"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const helpers = require("./postgres-runtime-helpers");
const { fanoutMapCall } = require("./postgres-fanout-calls");
const { reportUnanalyzable, unanalyzablePolicy } = require("./postgres-unanalyzable");

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
        unanalyzable:
          "This mapped query executor is not a static import, require, or import-equals binding. Write the module specifier as a string literal so the fan-out can be judged. A dynamic module is not a clean result.",
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
          const hit = callbackContainsExecutor(mapped.callback, bindings, context);
          if (hit === "unresolved") {
            reportUnanalyzable(context, node, unanalyzablePolicy(context.options?.[0]));
            return;
          }
          if (hit !== true) return;
          context.report({ node, messageId: "unboundedFanout" });
        },
      };
    },
  ),
  { __test: helpers },
);
