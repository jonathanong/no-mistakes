"use strict";

const { rule, selectorValueNode } = require("../helpers");
const { isDefaultedPropReference } = require("../defaulted-props");
const { ruleSuppression } = require("../rule-suppression");
const { selectorAttributeVisitors } = require("../selector-visitor");

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require literal defaults for prop-passed test IDs",
      recommended: true,
    },
    schema: [
      {
        type: "object",
        properties: { selectorAttributes: { type: "array", items: { type: "string" } } },
        additionalProperties: false,
      },
    ],
    messages: {
      default: "Test ID prop passthrough must have a literal default.",
      unanalyzable:
        "Cannot tell whether this JSX spread passes a test ID without a literal default. Write the test ID as a static attribute.",
    },
  },
  (context) => {
    const suppressed = ruleSuppression(context, "playwright-defaults");
    return selectorAttributeVisitors(
      context,
      (node) => {
        const valueNode = selectorValueNode(node);
        if (
          valueNode?.type === "Identifier" &&
          !isDefaultedPropReference(valueNode, context) &&
          !suppressed(node)
        ) {
          context.report({ node, messageId: "default" });
        }
      },
      (node, shape) => {
        if (shape.kind !== "opaque" && !shape.dynamicValue) return;
        if (!suppressed(node)) context.report({ node, messageId: "unanalyzable" });
      },
    );
  },
);
