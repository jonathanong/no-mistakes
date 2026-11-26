"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");

const TRANSPARENT_EXPRESSION_TYPES = new Set([
  "ChainExpression",
  "TSInstantiationExpression",
  "TSNonNullExpression",
]);

function exportName(node) {
  if (!node) return null;
  if (node.type === "Identifier") return node.name;
  return node.type === "Literal" ? String(node.value) : null;
}

function unwrapExpression(expression) {
  let current = expression;
  while (TRANSPARENT_EXPRESSION_TYPES.has(current.type)) current = current.expression;
  return current;
}

function staticName(node, computed) {
  if (!computed && node.type === "Identifier") return node.name;
  if (node.type === "Literal") return String(node.value);
  return null;
}

function isCommonJsBinding(identifier, context) {
  let scope = context.sourceCode.getScope(identifier);
  while (scope) {
    const variable = scope.set.get(identifier.name);
    if (variable?.defs.length) return false;
    scope = scope.upper;
  }
  return true;
}

function isExportsObject(node, context) {
  return node.type === "Identifier" && node.name === "exports" && isCommonJsBinding(node, context);
}

function isModuleExports(member, context) {
  if (member.type !== "MemberExpression") return false;
  if (staticName(member.property, member.computed) !== "exports") return false;
  const object = member.object;
  return (
    object.type === "Identifier" && object.name === "module" && isCommonJsBinding(object, context)
  );
}

// `undefined` is not a CommonJS export write. `null` is a write whose name is not static.
function cjsExportName(left, context) {
  if (left.type !== "MemberExpression") return undefined;
  if (!isExportsObject(left.object, context) && !isModuleExports(left.object, context)) {
    return undefined;
  }
  return staticName(left.property, left.computed);
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

function judgeImportEquals(node, report) {
  const target = node.moduleReference;
  if (target.type === "Identifier") {
    if (node.id.name !== target.name) report(node, "renamed");
    return;
  }
  report(node, "unanalyzable");
}

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: {
        description: "disallow value export renaming",
        recommended: true,
      },
      messages: {
        renamed:
          "Do not rename value exports. Export the original name or rename the declaration itself so agents can trace symbols directly.",
        unanalyzable:
          "Cannot tell whether this export renames a value. Assign or import one identifier.",
      },
      schema: [
        {
          type: "object",
          properties: {
            allowDefaultReExports: { type: "boolean" },
            includePathPatterns: { type: "array", items: { type: "string" } },
            unanalyzable: { enum: ["finding", "ignore"] },
          },
          additionalProperties: false,
        },
      ],
    },
    (context) => {
      const options = context.options[0] || {};
      const patterns = pathPatterns(options);
      if (!shouldCheckFile(context, options, patterns)) return {};
      const reportsUnanalyzable = (options.unanalyzable ?? "finding") === "finding";
      const suppressed = ruleSuppression(context, "ts-no-export-renaming");

      function report(node, messageId) {
        if (suppressed(node)) return;
        if (messageId === "unanalyzable" && !reportsUnanalyzable) return;
        context.report({ node, messageId });
      }

      return {
        ExportNamedDeclaration(node) {
          if (node.declaration?.type === "TSImportEqualsDeclaration") {
            judgeImportEquals(node.declaration, report);
          }
          for (const specifier of node.specifiers) {
            if (specifier.type !== "ExportSpecifier" || isTypeExport(node, specifier)) {
              continue;
            }
            if (options.allowDefaultReExports && isDefaultReExportAlias(node, specifier)) {
              continue;
            }
            const local = exportName(specifier.local);
            const exported = exportName(specifier.exported);
            if (local && exported && local !== exported) report(specifier, "renamed");
          }
        },
        AssignmentExpression(node) {
          if (node.operator !== "=") return;
          const exported = cjsExportName(node.left, context);
          if (exported === undefined) return;
          if (exported === null) {
            report(node, "unanalyzable");
            return;
          }
          const right = unwrapExpression(node.right);
          if (right.type !== "Identifier") {
            report(node, "unanalyzable");
            return;
          }
          if (exported !== right.name) report(node, "renamed");
        },
      };
    },
  ),
  { __test: { exportName } },
);
