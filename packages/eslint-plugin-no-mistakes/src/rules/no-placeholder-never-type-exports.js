"use strict";

const { rule } = require("../helpers");

function specifierName(specifier) {
  return specifier.local?.name || specifier.exported?.name || specifier.exported?.value;
}

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
  (context) => {
    const neverTypes = new Set();
    return {
      "Program > TSTypeAliasDeclaration"(node) {
        if (node.typeAnnotation?.type === "TSNeverKeyword") neverTypes.add(node.id.name);
      },
      ExportNamedDeclaration(node) {
        const declaration = node.declaration;
        if (declaration?.type === "TSTypeAliasDeclaration") {
          if (declaration.typeAnnotation?.type !== "TSNeverKeyword") return;
          context.report({ node, messageId: "placeholder" });
          return;
        }
        if (
          node.specifiers?.some(
            (specifier) =>
              (node.exportKind === "type" || specifier.exportKind === "type") &&
              neverTypes.has(specifierName(specifier)),
          )
        ) {
          context.report({ node, messageId: "placeholder" });
        }
      },
    };
  },
);
