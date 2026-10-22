"use strict";

const {
  cssSelectorValues,
  literalString,
  options,
  rule,
  selectorAttributes,
} = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { SELECTOR_METHODS, invokedCall, methodFromCallee } = require("./playwright-static-call");

module.exports = rule(
  {
    type: "suggestion",
    docs: { description: "prefer getByTestId over CSS test-id selectors", recommended: false },
    schema: [
      {
        type: "object",
        properties: { selectorAttributes: { type: "array", items: { type: "string" } } },
        additionalProperties: false,
      },
    ],
    messages: { prefer: "Prefer getByTestId('{{value}}') for exact test ID selectors." },
  },
  (context) => {
    const suppressed = ruleSuppression(context, "playwright-prefer-get-by-test-id");
    const attrs = selectorAttributes(options(context));
    return {
      CallExpression(node) {
        const invoked = invokedCall(node);
        let methodName = null;
        if (invoked) {
          methodName = methodFromCallee(invoked.callee, context);
        }
        if (!methodName || !SELECTOR_METHODS.has(methodName) || methodName === "getByTestId") {
          return;
        }
        for (const arg of invoked.arguments.slice(0, methodName === "dragAndDrop" ? 2 : 1)) {
          const source = literalString(arg);
          if (typeof source !== "string" || source === "") {
            continue;
          }
          for (const selector of cssSelectorValues(source, attrs)) {
            if (selector.operator === "=") {
              const hidden = suppressed(arg);
              if (hidden) {
                continue;
              }
              context.report({ node: arg, messageId: "prefer", data: { value: selector.value } });
            }
          }
        }
      },
    };
  },
);
