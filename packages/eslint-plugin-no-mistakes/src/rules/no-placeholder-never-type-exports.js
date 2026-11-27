"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");

function specifierName(specifier) {
  return specifier.local?.name || specifier.exported?.name || specifier.exported?.value;
}

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: { description: "disallow exported never placeholder type aliases", recommended: true },
      schema: [],
      messages: {
        placeholder:
          "Do not export placeholder never types. Define the real type or remove the export.",
      },
    },
    (context) => {
      const suppressed = ruleSuppression(context, "no-placeholder-never-type-exports");
      const neverTypes = new Set();
      const exportSpecifiers = [];
      function report(node) {
        if (!suppressed(node)) context.report({ node, messageId: "placeholder" });
      }
      return {
        "Program > TSTypeAliasDeclaration"(node) {
          if (node.typeAnnotation?.type === "TSNeverKeyword") neverTypes.add(node.id.name);
        },
        "Program:exit"() {
          for (const node of exportSpecifiers) {
            if (node.specifiers.some((specifier) => neverTypes.has(specifierName(specifier)))) {
              report(node);
            }
          }
        },
        ExportNamedDeclaration(node) {
          const declaration = node.declaration;
          if (declaration?.type === "TSTypeAliasDeclaration") {
            if (declaration.typeAnnotation?.type !== "TSNeverKeyword") return;
            report(node);
            return;
          }
          if (
            !node.source &&
            node.specifiers?.some(
              (specifier) => node.exportKind === "type" || specifier.exportKind === "type",
            )
          ) {
            exportSpecifiers.push(node);
          }
        },
      };
    },
  ),
  { __test: { specifierName } },
);
