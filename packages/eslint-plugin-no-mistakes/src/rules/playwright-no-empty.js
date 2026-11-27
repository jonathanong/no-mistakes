"use strict";

const { rule, selectorLiteral } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { selectorAttributeVisitors } = require("../selector-visitor");

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow empty test IDs", recommended: true },
    schema: [
      {
        type: "object",
        properties: { selectorAttributes: { type: "array", items: { type: "string" } } },
        additionalProperties: false,
      },
    ],
    messages: { empty: "Test ID must not be empty." },
  },
  (context) => {
    const suppressed = ruleSuppression(context, "playwright-no-empty");
    return selectorAttributeVisitors(context, (node) => {
      if (selectorLiteral(node) === "" && !suppressed(node)) {
        context.report({ node, messageId: "empty" });
      }
    });
  },
);
