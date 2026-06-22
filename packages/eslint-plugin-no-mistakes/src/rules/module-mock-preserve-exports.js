"use strict";

const { rule } = require("../helpers");
const { baselineKey, baselineSet } = require("./module-mock-baseline");
const {
  isInternalSpecifier,
  isPreserveMockCall,
  importSpecifierName,
  moduleMockSpecifierArgument,
  pathAllowed,
} = require("./module-mock-helpers");
const { createPreserveMockAliases } = require("./module-mock-preserve-aliases");
const { factoryPreservesExports } = require("./module-mock-preserve-factory");

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require module mock factories to preserve real exports",
      recommended: false,
    },
    schema: [{ type: "object" }],
    messages: {
      preserve: "Module mock factory must spread the real module to preserve exports.",
    },
  },
  (context) => {
    const options = context.options?.[0] ?? {};
    const baseline = baselineSet(options.baseline);
    const mockAliases = createPreserveMockAliases(context);
    if (!pathAllowed(context.filename, options)) return {};

    function reportMockCall(
      node,
      mock,
      specifierNode = node.arguments[0],
      factory = node.arguments[1],
    ) {
      if (!factory || factory.type === "ObjectExpression") return;
      const { dynamic, specifier } = moduleMockSpecifierArgument(specifierNode);
      if (dynamic || !specifier || !isInternalSpecifier(specifier, options)) return;
      if (baseline.has(baselineKey(context.filename, specifier))) return;
      if (!factoryPreservesExports(factory, specifier, mock, context)) {
        context.report({ node, messageId: "preserve" });
      }
    }

    return {
      VariableDeclarator(node) {
        mockAliases.declare(node.id, node.init);
      },
      ImportDeclaration(node) {
        if (node.source.value !== "vitest" && node.source.value !== "@jest/globals") return;
        for (const specifier of node.specifiers) {
          if (specifier.type !== "ImportSpecifier") continue;
          mockAliases.declareImport(
            specifier.local,
            node.source.value,
            importSpecifierName(specifier),
          );
        }
      },
      AssignmentExpression(node) {
        if (node.operator === "=") mockAliases.declare(node.left, node.right);
      },
      CallExpression(node) {
        const direct = isPreserveMockCall(node, context);
        if (direct) {
          reportMockCall(node, direct);
          return;
        }
        const alias = mockAliases.matchCall(node);
        if (alias) reportMockCall(node, alias.mock, alias.specifierNode, alias.factory);
      },
    };
  },
);
