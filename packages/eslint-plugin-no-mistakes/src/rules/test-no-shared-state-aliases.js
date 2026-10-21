"use strict";

const { importSpecifierName } = require("./test-no-shared-state-callees");

function resolveVariable(node, context) {
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === node.name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function baseIdentifier(callee) {
  if (callee?.type === "Identifier") return callee;
  if (callee?.type === "CallExpression") return baseIdentifier(callee.callee);
  if (callee?.type === "MemberExpression") return baseIdentifier(callee.object);
  return null;
}

function createImportedTestAliases(context) {
  const importedTestCallees = new Map();
  const describeAliases = new Set(["describe"]);
  return {
    add(local, imported) {
      importedTestCallees.set(local.name, resolveVariable(local, context));
      if (imported === "describe") describeAliases.add(local.name);
    },
    isDescribeAliasCall(callee) {
      const base = baseIdentifier(callee);
      return (
        base?.type === "Identifier" && describeAliases.has(base.name) && !this.isShadowed(callee)
      );
    },
    isShadowed(callee) {
      const base = baseIdentifier(callee);
      const imported = base ? importedTestCallees.get(base.name) : null;
      return Boolean(imported && resolveVariable(base, context) !== imported);
    },
    hasProperty(callee, name) {
      return hasProperty(callee, name);
    },
  };
}

function hasOwnProperty(callee, name) {
  return callee?.type === "MemberExpression" && !callee.computed && callee.property?.name === name;
}

// Recursively checks the whole callee chain (member accesses and call results) for a
// non-computed property named `name`, e.g. `hasProperty(test.describe.only, "describe")`.
function hasProperty(callee, name) {
  if (hasOwnProperty(callee, name)) return true;
  if (callee?.type === "MemberExpression") return hasProperty(callee.object, name);
  if (callee?.type === "CallExpression") return hasProperty(callee.callee, name);
  return false;
}

function recordImportedTests(node, testCalleeNames, namespaces, testAliases) {
  if (node.source.value !== "vitest" && node.source.value !== "@playwright/test") return;
  for (const specifier of node.specifiers) {
    if (specifier.type === "ImportNamespaceSpecifier" && specifier.local?.name) {
      namespaces.add(specifier.local.name);
    }
    if (
      specifier.type === "ImportDefaultSpecifier" &&
      node.source.value === "@playwright/test" &&
      specifier.local?.name
    ) {
      testCalleeNames.add(specifier.local.name);
      testAliases.add(specifier.local, "test");
    }
    if (specifier.type !== "ImportSpecifier") continue;
    const imported = importSpecifierName(specifier);
    if (["describe", "it", "test"].includes(imported) && specifier.local?.name) {
      testCalleeNames.add(specifier.local.name);
      testAliases.add(specifier.local, imported);
    }
  }
}

module.exports = {
  createImportedTestAliases,
  hasOwnProperty,
  hasProperty,
  recordImportedTests,
  resolveVariable,
};
