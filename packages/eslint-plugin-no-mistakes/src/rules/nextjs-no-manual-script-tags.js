"use strict";

const { rule } = require("../helpers");

module.exports = rule(
  {
    type: "problem",
    docs: { description: "prefer next/script over raw script JSX tags", recommended: false },
    schema: [],
    messages: { script: "Use next/script instead of a raw <script> tag." },
  },
  (context) => ({
    JSXOpeningElement(node) {
      if (node.name.type !== "JSXIdentifier" || node.name.name !== "script") return;
      context.report({ node, messageId: "script" });
    },
  }),
);
