"use strict";

const { rule } = require("../helpers");
const { baselineKey, baselineMap } = require("./module-mock-baseline");
const { matchDirectMockCallApply } = require("./module-mock-call-apply");
const { integrationAllows } = require("./module-mock-integration");
const {
  collectPatternNames,
  importSpecifierName,
  isFrameworkBinding,
  isInternalSpecifier,
  isModuleMockMemberCall,
  memberPropertyName,
  moduleMockSpecifierArgument,
  pathAllowed,
  propertyName,
  repoRelativeFilename,
} = require("./module-mock-helpers");

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

function resolveVariable(node, context) {
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === node.name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
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
    const mockAliases = new Map();
    if (!pathAllowed(filename, options)) return {};

    function declareAlias(id, init) {
      if (!id || !init) return;
      if (id.type === "ObjectPattern" && isFrameworkBinding(init, context)) {
        for (const property of id.properties) {
          if (property.type !== "Property") continue;
          const method = propertyName(property.key);
          if (MODULE_MOCK_METHODS.has(method)) {
            for (const name of collectPatternNames(property.value)) {
              mockAliases.set(name, resolveVariable(property.value, context));
            }
          }
        }
        return;
      }
      if (
        init.type === "MemberExpression" &&
        memberPropertyName(init) &&
        MODULE_MOCK_METHODS.has(memberPropertyName(init)) &&
        isFrameworkBinding(init.object, context)
      ) {
        for (const name of collectPatternNames(id))
          mockAliases.set(name, resolveVariable(id, context));
      }
    }

    function isUnshadowedMockAlias(node) {
      return (
        node.type === "Identifier" && mockAliases.get(node.name) === resolveVariable(node, context)
      );
    }

    function reportIfDisallowed(
      node,
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
      if (integrationAllows(specifier, factory, options)) return;
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
          if (MODULE_MOCK_METHODS.has(importSpecifierName(specifier))) {
            mockAliases.set(specifier.local.name, resolveVariable(specifier.local, context));
          }
        }
      },
      VariableDeclarator(node) {
        declareAlias(node.id, node.init);
      },
      AssignmentExpression(node) {
        if (node.operator === "=") declareAlias(node.left, node.right);
      },
      CallExpression(node) {
        const memberMock = isModuleMockMemberCall(node, context);
        if (memberMock && MODULE_MOCK_METHODS.has(memberMock.method)) {
          reportIfDisallowed(node);
          return;
        }
        const directCall = matchDirectMockCallApply(node, context, MODULE_MOCK_METHODS);
        if (directCall) {
          reportIfDisallowed(node, directCall.specifierNode, directCall.factory);
          return;
        }
        if (isUnshadowedMockAlias(node.callee)) {
          reportIfDisallowed(node);
          return;
        }
        if (
          node.callee.type === "MemberExpression" &&
          propertyName(node.callee.property) === "call" &&
          node.callee.object.type === "Identifier" &&
          isUnshadowedMockAlias(node.callee.object)
        ) {
          reportIfDisallowed(node, node.arguments[1], node.arguments[2]);
          return;
        }
        if (
          node.callee.type === "MemberExpression" &&
          propertyName(node.callee.property) === "apply" &&
          node.callee.object.type === "Identifier" &&
          isUnshadowedMockAlias(node.callee.object)
        ) {
          const args = node.arguments[1];
          reportIfDisallowed(
            node,
            args?.type === "ArrayExpression" ? args.elements[0] : undefined,
            args?.type === "ArrayExpression" ? args.elements[1] : undefined,
          );
        }
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
