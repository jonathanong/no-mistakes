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
  if (callee?.type === "ChainExpression") return baseIdentifier(callee.expression);
  if (callee?.type === "SequenceExpression" && callee.expressions.length > 0) {
    return baseIdentifier(callee.expressions.at(-1));
  }
  if (callee?.type === "Identifier") return callee;
  if (callee?.type === "CallExpression") return baseIdentifier(callee.callee);
  if (callee?.type === "MemberExpression") return baseIdentifier(callee.object);
  return null;
}

function createImportedTestAliases(context) {
  const importedTestCallees = new Map();
  const describeAliases = new Set(["describe"]);
  const serialAliases = new Set();
  return {
    add(local, imported, serial = false) {
      importedTestCallees.set(local.name, resolveVariable(local, context));
      if (imported === "describe") describeAliases.add(local.name);
      if (serial) serialAliases.add(local.name);
    },
    isDescribeAliasCall(callee) {
      const base = baseIdentifier(callee);
      return (
        base?.type === "Identifier" && describeAliases.has(base.name) && !this.isShadowed(callee)
      );
    },
    isSerialAlias(callee) {
      const base = baseIdentifier(callee);
      return (
        base?.type === "Identifier" && serialAliases.has(base.name) && !this.isShadowed(callee)
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
  if (callee?.type === "ChainExpression") return hasProperty(callee.expression, name);
  if (callee?.type === "SequenceExpression" && callee.expressions.length > 0) {
    return hasProperty(callee.expressions.at(-1), name);
  }
  if (hasOwnProperty(callee, name)) return true;
  if (callee?.type === "MemberExpression") return hasProperty(callee.object, name);
  if (callee?.type === "CallExpression") return hasProperty(callee.callee, name);
  return false;
}

function isDescribeBinding(
  node,
  testAliases,
  testCalleeNames,
  namespaces,
  calleeHasProperty,
  isKnownTestCallee,
) {
  let current = node;
  while (current?.type === "ChainExpression" || current?.type === "SequenceExpression") {
    current = current.type === "ChainExpression" ? current.expression : current.expressions.at(-1);
  }
  if (current?.type === "Identifier") {
    return current.name === "describe" || testAliases.isDescribeAliasCall(current);
  }
  return (
    current?.type === "MemberExpression" &&
    calleeHasProperty(current, "describe") &&
    isKnownTestCallee(current, testCalleeNames, namespaces)
  );
}

function recordImportedTests(node, testCalleeNames, namespaces, testAliases) {
  if (node.source.value !== "vitest" && node.source.value !== "@playwright/test") return;
  for (const specifier of node.specifiers) {
    if (specifier.type === "ImportNamespaceSpecifier" && specifier.local?.name) {
      namespaces.add(specifier.local.name);
      testAliases.add(specifier.local, "namespace");
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

function recordTopLevelBindings(node, tools) {
  const {
    calleeHasProperty,
    collectPatternNames,
    isKnownTestCallee,
    isMutableInitializer,
    isTestExtendCall,
    mutableTopLevel,
    namespaces,
    testAliases,
    testCalleeNames,
    viMockTracker,
  } = tools;
  for (const declaration of node.declarations) {
    if (isTestExtendCall(declaration.init, testCalleeNames, namespaces)) {
      for (const name of collectPatternNames(declaration.id)) testCalleeNames.add(name);
      if (declaration.id.type === "Identifier") testAliases.add(declaration.id, "test");
    }
    if (
      node.kind === "const" &&
      declaration.id.type === "Identifier" &&
      declaration.init &&
      declaration.init.type !== "CallExpression" &&
      isKnownTestCallee(declaration.init, testCalleeNames, namespaces)
    ) {
      testCalleeNames.add(declaration.id.name);
      if (
        isDescribeBinding(
          declaration.init,
          testAliases,
          testCalleeNames,
          namespaces,
          calleeHasProperty,
          isKnownTestCallee,
        )
      ) {
        testAliases.add(
          declaration.id,
          "describe",
          testAliases.hasProperty(declaration.init, "serial"),
        );
      } else {
        testAliases.add(declaration.id, "test");
      }
    }
    if (node.kind === "const" && !isMutableInitializer(declaration.init)) continue;
    for (const name of collectPatternNames(declaration.id)) {
      mutableTopLevel.add(name);
      viMockTracker.markIfCaptured(name);
    }
  }
}

module.exports = {
  createImportedTestAliases,
  hasOwnProperty,
  hasProperty,
  isDescribeBinding,
  recordImportedTests,
  recordTopLevelBindings,
  resolveVariable,
};
