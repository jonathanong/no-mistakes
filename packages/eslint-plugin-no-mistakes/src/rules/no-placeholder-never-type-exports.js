"use strict";

const { rule } = require("../helpers");

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow exported never placeholder type aliases", recommended: false },
    schema: [],
    messages: {
      placeholder:
        "Do not export placeholder never types. Define the real type or remove the export.",
    },
  },
  (context) => ({
    ExportNamedDeclaration(node) {
      const declaration = node.declaration;
      if (declaration?.type !== "TSTypeAliasDeclaration") return;
      if (declaration.typeAnnotation?.type !== "TSNeverKeyword") return;
      context.report({ node, messageId: "placeholder" });
    },
  }),
);
