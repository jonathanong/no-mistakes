"use strict";

const { rule } = require("../helpers");

const ALLOWED_SUFFIXES = [
  "/page.tsx",
  "/page.ts",
  "/page.jsx",
  "/page.js",
  "/layout.tsx",
  "/layout.ts",
  "/layout.jsx",
  "/layout.js",
];
const METADATA_EXPORTS = new Set(["metadata", "generateMetadata"]);
const NEXT_FILE_PATTERN = /(?:^|[/\\])(?:app|pages)(?:[/\\]|$)/;

function isAllowedFile(filename) {
  const normalized = filename.replace(/\\/g, "/");
  return (
    (normalized.startsWith("app/") || normalized.includes("/app/")) &&
    ALLOWED_SUFFIXES.some((suffix) => normalized.endsWith(suffix))
  );
}

function isNextFile(filename) {
  return NEXT_FILE_PATTERN.test(filename.replace(/\\/g, "/"));
}

function specifierName(specifier) {
  return specifier.exported?.name || specifier.exported?.value || specifier.local?.name;
}

function declarationName(declaration) {
  return declaration.id?.type === "Identifier" ? declaration.id.name : null;
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
      if (!isNextFile(context.filename)) return;
      if (isAllowedFile(context.filename)) return;
      if (node.declaration?.type === "VariableDeclaration") {
        if (
          node.declaration.declarations.some((declaration) =>
            METADATA_EXPORTS.has(declarationName(declaration)),
          )
        ) {
          context.report({ node, messageId: "location" });
        }
      }
      if (
        node.declaration?.type === "FunctionDeclaration" &&
        METADATA_EXPORTS.has(declarationName(node.declaration))
      ) {
        context.report({ node, messageId: "location" });
      }
      if (
        node.exportKind !== "type" &&
        node.specifiers?.some(
          (specifier) =>
            specifier.exportKind !== "type" && METADATA_EXPORTS.has(specifierName(specifier)),
        )
      ) {
        context.report({ node, messageId: "location" });
      }
    },
  }),
);
