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

function collectPatternNames(node, names = new Set()) {
  if (!node) return names;
  if (node.type === "Identifier") names.add(node.name);
  if (node.type === "ObjectPattern") {
    for (const property of node.properties) collectPatternNames(property.value, names);
  }
  if (node.type === "ArrayPattern") {
    for (const element of node.elements) collectPatternNames(element, names);
  }
  if (node.type === "RestElement") collectPatternNames(node.argument, names);
  return names;
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
            [...collectPatternNames(declaration.id), declarationName(declaration)].some((name) =>
              METADATA_EXPORTS.has(name),
            ),
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
