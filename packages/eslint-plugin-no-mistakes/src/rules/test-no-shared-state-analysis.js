"use strict";

const {
  childNodes,
  isCalledFunction,
  isFunctionNode,
  isInlineSetupCallback,
  isInlineTestCallback,
} = require("./test-no-shared-state-helpers");

function isInsideUncalledNestedFunction(node, testDepth, setupDepth) {
  if (testDepth === 0 && setupDepth === 0) return false;
  let current = node.parent;
  while (current) {
    const isUncalledFunction =
      isFunctionNode(current) &&
      !isInlineSetupCallback(current) &&
      !isInlineTestCallback(current) &&
      !isCalledFunction(current);
    if (isUncalledFunction) return true;
    current = current.parent;
  }
  return false;
}

function isModuleMutable({ context, mutableTopLevel, node, name }) {
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const get = scope.set?.get;
    const variable = typeof get === "function" ? get.call(scope.set, name) : null;
    const resolvedVariable =
      variable ||
      (typeof get !== "function" ? scope.variables?.find((item) => item.name === name) : null);

    if (!resolvedVariable) {
      scope = scope.upper;
      continue;
    }

    return (
      mutableTopLevel.has(resolvedVariable.name) &&
      (resolvedVariable.scope.type === "module" || resolvedVariable.scope.block.type === "Program")
    );
  }
  return false;
}

function isResetAssignment(node, isMutableInitializer) {
  if (node.left.type === "Identifier") return isMutableInitializer(node.right);
  return (
    node.left.type === "MemberExpression" &&
    !node.left.computed &&
    node.left.property.type === "Identifier" &&
    node.left.property.name === "length" &&
    node.right.type === "Literal" &&
    node.right.value === 0
  );
}

function walkSharedMutations(node, handlers) {
  if (isFunctionNode(node) && !isCalledFunction(node)) return;
  if (node.type === "AssignmentExpression") {
    handlers.onAssignment(node);
  } else if (node.type === "UpdateExpression") {
    handlers.onUpdate(node);
  } else if (node.type === "CallExpression") {
    handlers.onCall(node);
  }
  for (const child of childNodes(node)) walkSharedMutations(child, handlers);
}

function createViMockTracker(context, mutableTopLevel) {
  const factoryReferences = new Set();
  const capturedMutables = new Set();

  function markIfCaptured(name) {
    if (name.startsWith("mock") && mutableTopLevel.has(name) && factoryReferences.has(name)) {
      capturedMutables.add(name);
    }
  }

  return {
    collectFactoryReferences(node) {
      if (
        node.callee.type !== "MemberExpression" ||
        node.callee.object.type !== "Identifier" ||
        node.callee.object.name !== "vi" ||
        node.callee.property.type !== "Identifier" ||
        node.callee.property.name !== "mock"
      ) {
        return;
      }
      const factory = node.arguments[1];
      if (!isFunctionNode(factory)) return;
      for (const { identifier } of context.sourceCode.getScope(factory).through) {
        factoryReferences.add(identifier.name);
        markIfCaptured(identifier.name);
      }
    },
    isCaptured(name) {
      return capturedMutables.has(name);
    },
    markIfCaptured,
  };
}

function createRegistryReports(context, mutableTopLevel, cleanupTracker, isCaptured) {
  const pending = [];

  return {
    flush() {
      for (const { node, path, suiteKey } of pending) {
        if (!cleanupTracker.has(path, suiteKey)) context.report({ node, messageId: "shared" });
      }
    },
    remember(node, name, path, testDepth, setupDepth) {
      if (
        name &&
        testDepth > 0 &&
        setupDepth === 0 &&
        !isCaptured(name) &&
        isModuleMutable({ context, mutableTopLevel, node, name })
      ) {
        pending.push({ node, path, suiteKey: cleanupTracker.currentSuiteKey() });
      }
    },
  };
}

module.exports = {
  createRegistryReports,
  createViMockTracker,
  isInsideUncalledNestedFunction,
  isModuleMutable,
  isResetAssignment,
  walkSharedMutations,
};
