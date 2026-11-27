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
        unanalyzable:
          "Cannot tell whether this exported type is a never placeholder. Its definition is in another module this file cannot read.",
      },
    },
    (context) => {
      const suppressed = ruleSuppression(context, "no-placeholder-never-type-exports");
      const declared = new Map();
      const exportSpecifiers = [];
      function report(node, messageId) {
        if (!suppressed(node)) context.report({ node, messageId });
      }
      function isTypeExport(node) {
        return (
          node.exportKind === "type" ||
          node.specifiers?.some((specifier) => specifier.exportKind === "type")
        );
      }
      return {
        "Program > TSTypeAliasDeclaration"(node) {
          declared.set(
            node.id.name,
            node.typeAnnotation?.type === "TSNeverKeyword" ? "never" : "other",
          );
        },
        "Program > TSInterfaceDeclaration"(node) {
          if (node.id?.name) declared.set(node.id.name, "other");
        },
        "Program:exit"() {
          for (const node of exportSpecifiers) {
            let never = false;
            let unread = false;
            for (const specifier of node.specifiers) {
              if (node.exportKind !== "type" && specifier.exportKind !== "type") continue;
              const kind = declared.get(specifierName(specifier));
              if (kind === "never") never = true;
              else if (kind !== "other") unread = true;
            }
            if (never) report(node, "placeholder");
            if (unread) report(node, "unanalyzable");
          }
        },
        ExportAllDeclaration(node) {
          if (node.exportKind === "type") report(node, "unanalyzable");
        },
        ExportNamedDeclaration(node) {
          const declaration = node.declaration;
          if (declaration?.type === "TSTypeAliasDeclaration") {
            if (declaration.typeAnnotation?.type !== "TSNeverKeyword") return;
            report(node, "placeholder");
            return;
          }
          if (!isTypeExport(node) || !node.specifiers?.length) return;
          if (node.source) {
            report(node, "unanalyzable");
            return;
          }
          exportSpecifiers.push(node);
        },
      };
    },
  ),
  { __test: { specifierName } },
);
