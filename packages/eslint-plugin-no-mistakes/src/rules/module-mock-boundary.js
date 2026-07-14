"use strict";

const { rule } = require("../helpers");
const { baselineKey, baselineMap } = require("./module-mock-baseline");
const { matchDirectMockCallApply } = require("./module-mock-call-apply");
const { integrationAllows } = require("./module-mock-integration");
const {
  importSpecifierName,
  isInternalSpecifier,
  isModuleMockMemberCall,
  moduleMockSpecifierArgument,
  pathAllowed,
  repoRelativeFilename,
} = require("./module-mock-helpers");
const { createMockAliases } = require("./module-mock-preserve-aliases");

const MODULE_MOCK_METHODS = new Set([
  "doMock",
  "importMock",
  "mock",
  "setMock",
  "unstable_mockModule",
]);

function createBaselineTracker(filename, entries) {
  const normalizedFilename = repoRelativeFilename(filename);
  const baseline = baselineMap(entries);
  const seen = new Map();
  return {
    allowed(specifier) {
      const key = baselineKey(filename, specifier);
      const count = (seen.get(key) ?? 0) + 1;
      seen.set(key, count);
      return count <= (baseline.get(key) ?? 0);
    },
    stale() {
      const stale = [];
      for (const [key, allowed] of baseline) {
        const [file, specifier] = JSON.parse(key);
        if (file !== normalizedFilename) continue;
        const count = seen.get(key) ?? 0;
        if (count < allowed) stale.push({ specifier, allowed, seen: count });
      }
      return stale;
    },
  };
}

function reportMessage(dynamic, specifier) {
  if (dynamic) return "Module mock boundary requires literal specifiers.";
  return `Module mock boundary does not allow mocking internal module "${specifier}".`;
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "enforce configured module mock boundaries",
      recommended: false,
    },
    schema: [{ type: "object" }],
    messages: {
      boundary: "{{message}}",
      stale: "{{message}}",
    },
  },
  (context) => {
    const options = context.options?.[0] ?? {};
    const filename = context.filename;
    const tracker = createBaselineTracker(filename, options.baseline);
    const mockAliases = createMockAliases(context, MODULE_MOCK_METHODS);
    if (!pathAllowed(filename, options)) return {};

    function reportIfDisallowed(
      node,
      mock,
      specifierNode = node.arguments[0],
      factory = node.arguments[1],
    ) {
      const { dynamic, specifier } = moduleMockSpecifierArgument(specifierNode);
      if (dynamic) {
        if (options.requireLiteralSpecifiers === false) return;
        context.report({ node, messageId: "boundary", data: { message: reportMessage(true) } });
        return;
      }
      if (!specifier || !isInternalSpecifier(specifier, options)) return;
      if (integrationAllows(specifier, factory, mock, context, options)) return;
      if (tracker.allowed(specifier)) return;
      context.report({
        node,
        messageId: "boundary",
        data: { message: reportMessage(false, specifier) },
      });
    }

    return {
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
      VariableDeclarator(node) {
        mockAliases.declare(node.id, node.init);
      },
      AssignmentExpression(node) {
        if (node.operator === "=") mockAliases.declare(node.left, node.right);
      },
      CallExpression(node) {
        const memberMock = isModuleMockMemberCall(node, context);
        if (memberMock && MODULE_MOCK_METHODS.has(memberMock.method)) {
          reportIfDisallowed(node, memberMock);
          return;
        }
        const directCall = matchDirectMockCallApply(node, context, MODULE_MOCK_METHODS);
        if (directCall) {
          reportIfDisallowed(node, directCall.mock, directCall.specifierNode, directCall.factory);
          return;
        }
        const alias = mockAliases.matchCall(node);
        if (alias) reportIfDisallowed(node, alias.mock, alias.specifierNode, alias.factory);
      },
      "Program:exit"(node) {
        for (const entry of tracker.stale()) {
          context.report({
            node,
            messageId: "stale",
            data: {
              message: `Module mock boundary baseline for "${entry.specifier}" is stale; lower count from ${entry.allowed} to ${entry.seen}.`,
            },
          });
        }
      },
    };
  },
);
