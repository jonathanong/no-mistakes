"use strict";

const { rule } = require("../helpers");

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow fixed sleeps in Playwright tests", recommended: false },
    schema: [],
    messages: {
      timeout:
        "Do not use setTimeout() in Playwright tests. Wait for an observable condition instead.",
    },
  },
  (context) => ({
    CallExpression(node) {
      if (node.callee.type !== "Identifier" || node.callee.name !== "setTimeout") return;
      context.report({ node, messageId: "timeout" });
    },
  }),
);
