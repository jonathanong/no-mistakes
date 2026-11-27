"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");

const TRANSPARENT_EXPRESSION_TYPES = new Set([
  "ChainExpression",
  "TSInstantiationExpression",
  "TSNonNullExpression",
]);

function unwrapExpression(expression) {
  let current = expression;
  while (TRANSPARENT_EXPRESSION_TYPES.has(current.type)) {
    current = current.expression;
  }
  return current;
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow const aliases for differently named values",
      recommended: true,
    },
    schema: [
      {
        type: "object",
        properties: {
          unanalyzable: { enum: ["finding", "ignore"] },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      alias:
        "Do not create a differently named const alias. Use the original name directly so readers and tooling can trace symbols without indirection.",
      unanalyzable:
        "Cannot tell whether this import aliases a different name. Bind it to one identifier.",
    },
  },
  (context) => {
    const reportsUnanalyzable = (context.options[0]?.unanalyzable ?? "finding") === "finding";
    const suppressed = ruleSuppression(context, "ts-no-const-aliases");

    function report(node, messageId) {
      if (suppressed(node)) return;
      if (messageId === "unanalyzable" && !reportsUnanalyzable) return;
      context.report({ node, messageId });
    }

    return {
      VariableDeclaration(node) {
        if (node.kind !== "const") return;
        for (const declarator of node.declarations) {
          if (
            declarator.id.type !== "Identifier" ||
            declarator.id.typeAnnotation ||
            !declarator.init
          )
            continue;
          const initializer = unwrapExpression(declarator.init);
          if (initializer.type === "Identifier" && declarator.id.name !== initializer.name) {
            report(declarator, "alias");
          }
        }
      },
      TSImportEqualsDeclaration(node) {
        if (node.parent?.type === "ExportNamedDeclaration") return;
        const target = node.moduleReference;
        if (target.type === "Identifier") {
          if (node.id.name !== target.name) report(node, "alias");
          return;
        }
        report(node, "unanalyzable");
      },
    };
  },
);
