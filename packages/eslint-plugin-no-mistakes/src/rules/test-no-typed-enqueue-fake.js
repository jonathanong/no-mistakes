"use strict";

const { posix } = require("node:path");
const { options, rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { memberPropertyName } = require("./logical-call");
const { importSpecifierName } = require("./module-mock-helpers");

const id = "test-no-typed-enqueue-fake";
const frameworkModules = new Map([
  ["vi", "vitest"],
  ["jest", "@jest/globals"],
]);

function referenceBinding(identifier, sourceCode) {
  const scope = sourceCode.getScope(identifier);
  return scope.references.find((reference) => reference.identifier === identifier)?.resolved;
}

function importDefinition(binding) {
  const def = binding?.defs.length === 1 ? binding.defs[0] : null;
  return def?.type === "ImportBinding" && def.parent?.type === "ImportDeclaration" ? def : null;
}

function runtimeImport(def) {
  return def && def.node.importKind !== "type" && def.parent.importKind !== "type";
}

function isFrameworkReceiver(receiver, sourceCode) {
  if (receiver.type === "Identifier") {
    const binding = referenceBinding(receiver, sourceCode);
    if (!binding?.defs.length) return frameworkModules.has(receiver.name);
    const def = importDefinition(binding);
    return (
      runtimeImport(def) &&
      frameworkModules.get(importSpecifierName(def.node)) === def.parent.source.value
    );
  }
  if (receiver.type !== "MemberExpression" || receiver.object.type !== "Identifier") return false;
  const def = importDefinition(referenceBinding(receiver.object, sourceCode));
  return (
    runtimeImport(def) &&
    def.node.type === "ImportNamespaceSpecifier" &&
    frameworkModules.get(memberPropertyName(receiver)) === def.parent.source.value
  );
}

function exportPattern(pattern) {
  if (pattern === undefined) return null;
  try {
    return new RegExp(pattern);
  } catch (cause) {
    throw new Error(`${id}: singleExportPattern must be a valid regular expression`, { cause });
  }
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require enqueue fakes to declare their argument tuple explicitly",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          singleExportPattern: { type: "string", pattern: "\\S" },
          moduleGlobs: { type: "array", items: { type: "string", pattern: "\\S" } },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      typedEnqueueFake:
        "Type the enqueue fake with an explicit argument tuple, or use the real function with a mocked queue.",
    },
  },
  (context) => {
    const { singleExportPattern, moduleGlobs = [] } = options(context);
    const exportMatcher = exportPattern(singleExportPattern);
    if (!exportMatcher && moduleGlobs.length === 0) return {};
    const { sourceCode } = context;
    const suppressed = ruleSuppression(context, id);
    return {
      CallExpression(node) {
        const types = node.typeArguments?.params;
        const query = types?.[0];
        if (
          types?.length !== 1 ||
          query.type !== "TSTypeQuery" ||
          query.exprName.type !== "Identifier"
        )
          return;
        const { callee } = node;
        if (
          callee.type !== "MemberExpression" ||
          memberPropertyName(callee) !== "fn" ||
          !isFrameworkReceiver(callee.object, sourceCode)
        )
          return;
        const target = importDefinition(referenceBinding(query.exprName, sourceCode));
        if (!target || target.node.type === "ImportNamespaceSpecifier") return;
        const name = importSpecifierName(target.node) ?? "default";
        if (exportMatcher && !exportMatcher.test(name)) return;
        if (
          moduleGlobs.length > 0 &&
          !moduleGlobs.some((glob) => posix.matchesGlob(target.parent.source.value, glob))
        ) {
          return;
        }
        if (!suppressed(node)) context.report({ node, messageId: "typedEnqueueFake" });
      },
    };
  },
);
