"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { preserveOptions } = require("./module-mock-schema");
const { reportUnanalyzable } = require("./rule-unanalyzable");
const { baselineKey, baselineSet } = require("./module-mock-baseline");
const { matchDirectMockCallApply } = require("./module-mock-call-apply");
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
    schema: [preserveOptions],
    messages: {
      preserve: "Module mock factory must spread the real module to preserve exports.",
      unanalyzable:
        "Cannot tell whether {{subject}} is a framework mock. Use a static vitest or Jest import.",
    },
  },
  (context) => {
    const options = context.options?.[0] ?? {};
    const baseline = baselineSet(options.baseline);
    const mockAliases = createPreserveMockAliases(context);
    if (!pathAllowed(context.filename, options)) return {};
    const suppressed = ruleSuppression(context, "module-mock-preserve-exports");

    function reportMockCall(
      node,
      mock,
      specifierNode = node.arguments[0],
      factory = node.arguments[1],
    ) {
      if (suppressed(node)) return;
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
        if (direct?.unanalyzable) {
          if (!suppressed(node)) reportUnanalyzable(context, node, "this call");
          return;
        }
        if (direct) {
          reportMockCall(node, direct);
          return;
        }
        const forwarded = matchDirectMockCallApply(
          node,
          context,
          new Set(["mock", "doMock", "unstable_mockModule"]),
        );
        if (forwarded?.mock.unanalyzable) {
          if (!suppressed(node)) reportUnanalyzable(context, node, "this call");
          return;
        }
        if (forwarded) {
          reportMockCall(
            node,
            forwarded.mock,
            forwarded.specifierNode ?? null,
            forwarded.factory ?? null,
          );
          return;
        }
        const alias = mockAliases.matchCall(node);
        if (!alias) return;
        if (alias.forwarded) {
          reportMockCall(node, alias.mock, alias.specifierNode, alias.factory);
          return;
        }
        reportMockCall(node, alias.mock);
      },
    };
  },
);
