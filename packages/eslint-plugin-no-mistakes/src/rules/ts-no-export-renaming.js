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

function pathPatterns(options) {
  return (options.includePathPatterns || []).flatMap((pattern) => {
    try {
      return [new RegExp(pattern)];
    } catch {
      return [];
    }
  });
}

function normalizedPaths(context) {
  const normalized = context.filename.replace(/\\/g, "/");
  const cwd = context.cwd?.replace(/\\/g, "/");
  if (cwd && normalized.startsWith(`${cwd}/`)) {
    return [normalized.slice(cwd.length + 1), normalized];
  }
  return [normalized];
}

function shouldCheckFile(context, options, patterns) {
  if ((options.includePathPatterns || []).length > 0 && patterns.length === 0) return false;
  if (patterns.length === 0) return true;
  return normalizedPaths(context).some((path) => patterns.some((pattern) => pattern.test(path)));
}

function isDefaultReExportAlias(node, specifier) {
  return exportName(specifier.local) === "default" && Boolean(node.source);
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow value export renaming",
      recommended: true,
    },
    messages: {
      renamed:
        "Do not rename value exports. Export the original name or rename the declaration itself so agents can trace symbols directly.",
    },
    schema: [
      {
        type: "object",
        properties: {
          allowDefaultReExports: { type: "boolean" },
          includePathPatterns: { type: "array", items: { type: "string" } },
        },
        additionalProperties: false,
      },
    ],
  },
  (context) => {
    const options = context.options[0] || {};
    const patterns = pathPatterns(options);
    if (!shouldCheckFile(context, options, patterns)) return {};
    return {
      ExportNamedDeclaration(node) {
        for (const specifier of node.specifiers || []) {
          if (specifier.type !== "ExportSpecifier" || isTypeExport(node, specifier)) {
            continue;
          }
          if (options.allowDefaultReExports && isDefaultReExportAlias(node, specifier)) {
            continue;
          }
          const local = exportName(specifier.local);
          const exported = exportName(specifier.exported);
          if (local && exported && local !== exported) {
            context.report({ node: specifier, messageId: "renamed" });
          }
        }
      },
    };
  },
);
