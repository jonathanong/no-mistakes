"use strict";

const { rule } = require("../helpers");

const ALLOWED_SUFFIXES = [
  "/page.tsx",
  "/page.ts",
  "/page.jsx",
  "/layout.tsx",
  "/layout.ts",
  "/layout.jsx",
  "/template.tsx",
  "/template.ts",
  "/template.jsx",
  "/default.tsx",
  "/default.ts",
  "/default.jsx",
];

function isAllowedFile(filename) {
  return ALLOWED_SUFFIXES.some((suffix) => filename.replace(/\\/g, "/").endsWith(suffix));
}

function specifierName(specifier) {
  return specifier.local?.name || specifier.exported?.name || specifier.exported?.value;
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "restrict Next.js metadata exports to route segment files",
      recommended: false,
    },
    schema: [],
    messages: {
      location:
        "metadata and generateMetadata exports are only allowed in Next.js route segment files.",
    },
  },
  (context) => ({
    ExportNamedDeclaration(node) {
      if (isAllowedFile(context.filename)) return;
      if (node.declaration?.type === "VariableDeclaration") {
        if (
          node.declaration.declarations.some((declaration) => declaration.id?.name === "metadata")
        ) {
          context.report({ node, messageId: "location" });
        }
      }
      if (
        node.declaration?.type === "FunctionDeclaration" &&
        node.declaration.id?.name === "generateMetadata"
      ) {
        context.report({ node, messageId: "location" });
      }
      if (node.specifiers?.some((specifier) => specifierName(specifier) === "metadata")) {
        context.report({ node, messageId: "location" });
      }
    },
  }),
);
