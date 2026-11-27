"use strict";

const { canonicalAttribute, options, rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { selectorAttributeVisitors } = require("../selector-visitor");

module.exports = rule(
  {
    type: "suggestion",
    docs: { description: "require a canonical test ID attribute", recommended: false },
    schema: [
      {
        type: "object",
        properties: {
          selectorAttributes: { type: "array", items: { type: "string" } },
          canonicalAttribute: { type: "string" },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      attribute: "Use '{{expected}}' instead of '{{actual}}' for test IDs.",
      unanalyzable:
        "Cannot tell whether this JSX spread or namespaced name sets a non-canonical test ID. Write the test ID as a static attribute.",
    },
  },
  (context) => {
    const suppressed = ruleSuppression(context, "playwright-consistent-attribute");
    return selectorAttributeVisitors(
      context,
      (node, name) => {
        const expected = canonicalAttribute(options(context));
        if (name !== expected && !suppressed(node.name)) {
          context.report({
            node: node.name,
            messageId: "attribute",
            data: { actual: name, expected },
          });
        }
      },
      (node, shape) => {
        if (shape.kind !== "namespaced" && shape.kind !== "opaque" && !shape.nonCanonical) return;
        if (!suppressed(node)) context.report({ node, messageId: "unanalyzable" });
      },
    );
  },
);
