"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");

function specifierName(specifier) {
  return specifier.local?.name || specifier.exported?.name || specifier.exported?.value;
}

function declaredType(scope, name) {
  while (scope) {
    const definitions = scope.set.get(name)?.defs;
    if (definitions?.length) {
      const localType = definitions.find(
        (definition) => definition.isTypeDefinition && definition.type !== "ImportBinding",
      );
      if (localType) {
        const { node } = localType;
        return node.type === "TSTypeAliasDeclaration" &&
          node.typeAnnotation?.type === "TSNeverKeyword"
          ? "never"
          : "other";
      }
      return definitions.some((definition) => definition.type === "ImportBinding")
        ? undefined
        : "other";
    }
    scope = scope.upper;
  }
  return undefined;
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
      const { sourceCode } = context;
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
        "Program:exit"() {
          for (const node of exportSpecifiers) {
            let never = false;
            let unread = false;
            for (const specifier of node.specifiers) {
              const kind = declaredType(sourceCode.getScope(specifier), specifierName(specifier));
              if (kind === "never") never = true;
              else if (
                kind === undefined &&
                (node.exportKind === "type" || specifier.exportKind === "type")
              )
                unread = true;
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
