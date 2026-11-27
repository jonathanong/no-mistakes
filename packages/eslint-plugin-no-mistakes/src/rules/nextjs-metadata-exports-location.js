"use strict";

const { literalString, rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");

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

function isFree(identifier, context) {
  let scope = context.sourceCode.getScope(identifier);
  while (scope) {
    const variable = scope.variables?.find((item) => item.name === identifier.name);
    if (variable) return variable.defs.length === 0;
    scope = scope.upper;
  }
  return true;
}

function isExportsBinding(node, context) {
  return node?.type === "Identifier" && node.name === "exports" && isFree(node, context);
}

function isModuleExports(node, context) {
  return (
    node?.type === "MemberExpression" &&
    !node.computed &&
    node.property?.type === "Identifier" &&
    node.property.name === "exports" &&
    node.object?.type === "Identifier" &&
    node.object.name === "module" &&
    isFree(node.object, context)
  );
}

function propertyStatus(key, computed) {
  if (!computed && key?.type === "Identifier") return { kind: "static", name: key.name };
  const name = literalString(key);
  if (name !== null) return { kind: "static", name };
  return { kind: "dynamic" };
}

module.exports = Object.assign(
  rule(
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
        unanalyzable:
          "Could not resolve whether this CommonJS assignment exports metadata or generateMetadata. Use a static property name.",
      },
    },
    (context) => {
      const suppressed = ruleSuppression(context, "nextjs-metadata-exports-location");
      const checked = isNextFile(context.filename) && !isAllowedFile(context.filename);
      function report(node) {
        if (!suppressed(node)) context.report({ node, messageId: "location" });
      }
      function reportProperty(key, computed, node) {
        const status = propertyStatus(key, computed);
        if (status.kind === "dynamic") {
          if (!suppressed(node)) context.report({ node, messageId: "unanalyzable" });
          return;
        }
        if (METADATA_EXPORTS.has(status.name)) report(node);
      }
      return {
        ExportNamedDeclaration(node) {
          if (!checked) return;
          if (node.declaration?.type === "VariableDeclaration") {
            if (
              node.declaration.declarations.some((declaration) =>
                [...collectPatternNames(declaration.id), declarationName(declaration)].some(
                  (name) => METADATA_EXPORTS.has(name),
                ),
              )
            ) {
              report(node);
            }
          }
          if (
            node.declaration?.type === "FunctionDeclaration" &&
            METADATA_EXPORTS.has(declarationName(node.declaration))
          ) {
            report(node);
          }
          if (
            node.declaration?.type === "TSImportEqualsDeclaration" &&
            node.declaration.id?.type === "Identifier" &&
            METADATA_EXPORTS.has(node.declaration.id.name)
          ) {
            report(node);
          }
          if (
            node.exportKind !== "type" &&
            node.specifiers?.some(
              (specifier) =>
                specifier.exportKind !== "type" && METADATA_EXPORTS.has(specifierName(specifier)),
            )
          ) {
            report(node);
          }
        },
        AssignmentExpression(node) {
          if (!checked || node.left.type !== "MemberExpression") return;
          const left = node.left;
          if (isExportsBinding(left.object, context) || isModuleExports(left.object, context)) {
            reportProperty(left.property, left.computed, left);
            return;
          }
          if (!isModuleExports(left, context) || node.right.type !== "ObjectExpression") return;
          for (const property of node.right.properties) {
            if (property.type === "Property")
              reportProperty(property.key, property.computed, property);
          }
        },
      };
    },
  ),
  { __test: { specifierName, collectPatternNames } },
);
