"use strict";

const { rule } = require("../helpers");

function exportName(node) {
  if (!node) return null;
  if (node.type === "Identifier") return node.name;
  return node.type === "Literal" ? String(node.value) : null;
}

function isTypeExport(node, specifier) {
  return node.exportKind === "type" || specifier.exportKind === "type";
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow value export renaming",
      recommended: true,
    },
    schema: [],
    messages: {
      renamed:
        "Do not rename value exports. Export the original name or rename the declaration itself so agents can trace symbols directly.",
    },
  },
  (context) => ({
    ExportNamedDeclaration(node) {
      for (const specifier of node.specifiers || []) {
        if (specifier.type !== "ExportSpecifier" || isTypeExport(node, specifier)) {
          continue;
        }
        const local = exportName(specifier.local);
        const exported = exportName(specifier.exported);
        if (local && exported && local !== exported) {
          context.report({ node: specifier, messageId: "renamed" });
        }
      }
    },
  }),
);
