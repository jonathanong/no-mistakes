"use strict";

const { rule } = require("../helpers");
const {
  childNodes,
  collectPatternNames,
  isCalledFunction,
  isFunctionNode,
  isInlineTestCallback,
  isMutableInitializer,
  isSetupCall,
  isTestCall,
  mutatingCallRootName,
  mutationRootName,
  namedCallbackArgument,
} = require("./test-no-shared-state-helpers");

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow mutable module-scope test state", recommended: false },
    schema: [],
    messages: {
      shared:
        "Shared mutable module-scope state between tests: use local variables inside each test instead.",
    },
  },
  (context) => {
    const mutableTopLevel = new Set();
    const functionDeclarations = new Map();
    const pendingNamedCallbacks = [];
    const viMockFactoryReferences = new Set();
    const viMockCapturedMutables = new Set();
    let testDepth = 0;
    let setupDepth = 0;

    function markViMockCapturedMutable(name) {
      if (
        /^mock[A-Z0-9_]/.test(name) &&
        mutableTopLevel.has(name) &&
        viMockFactoryReferences.has(name)
      ) {
        viMockCapturedMutables.add(name);
      }
    }

    function isModuleMutable(node, name) {
      let scope = context.sourceCode.getScope(node);
      while (scope) {
        const variable = scope.variables.find((candidate) => candidate.name === name);
        if (variable) {
          return (
            mutableTopLevel.has(variable.name) &&
            (variable.scope.type === "module" || variable.scope.block.type === "Program")
          );
        }
        scope = scope.upper;
      }
      return false;
    }

    function reportIfShared(node, name) {
      if (
        name &&
        testDepth > 0 &&
        setupDepth === 0 &&
        !viMockCapturedMutables.has(name) &&
        isModuleMutable(node, name)
      ) {
        context.report({ node, messageId: "shared" });
      }
    }

    function reportAssignment(node) {
      for (const name of collectPatternNames(node.left)) reportIfShared(node, name);
      if (node.left.type !== "MemberExpression") return;
      reportIfShared(node, mutationRootName(node.left));
    }

    return {
      "Program > VariableDeclaration"(node) {
        for (const declaration of node.declarations) {
          if (declaration.id.type === "Identifier" && isFunctionNode(declaration.init)) {
            functionDeclarations.set(declaration.id.name, declaration.init);
          }
          if (node.kind === "const" && !isMutableInitializer(declaration.init)) continue;
          for (const name of collectPatternNames(declaration.id)) {
            mutableTopLevel.add(name);
            markViMockCapturedMutable(name);
          }
        }
      },
      "Program > FunctionDeclaration"(node) {
        if (node.id?.name) functionDeclarations.set(node.id.name, node);
      },
      "Program:exit"() {
        testDepth = 1;
        for (const name of pendingNamedCallbacks) {
          const declaration = functionDeclarations.get(name);
          if (declaration) checkSharedMutations(declaration.body);
        }
      },
      CallExpression(node) {
        collectViMockFactoryReferences(node);
        if (isTestCall(node)) {
          testDepth += 1;
          const callback = namedCallbackArgument(node.arguments);
          if (callback) pendingNamedCallbacks.push(callback.name);
        }
        if (isSetupCall(node)) setupDepth += 1;
      },
      AssignmentExpression(node) {
        if (isInsideUncalledNestedFunction(node)) return;
        reportAssignment(node);
      },
      UpdateExpression(node) {
        if (isInsideUncalledNestedFunction(node)) return;
        reportIfShared(node, mutationRootName(node.argument));
      },
      "CallExpression:exit"(node) {
        if (isTestCall(node)) testDepth -= 1;
        if (isSetupCall(node)) setupDepth -= 1;
        if (isInsideUncalledNestedFunction(node)) return;
        reportIfShared(node, mutatingCallRootName(node));
      },
    };

    function isInsideUncalledNestedFunction(node) {
      if (testDepth === 0) return false;
      let current = node.parent;
      while (current) {
        const isUncalledFunction =
          isFunctionNode(current) && !isInlineTestCallback(current) && !isCalledFunction(current);
        if (isUncalledFunction) return true;
        current = current.parent;
      }
      return false;
    }

    function checkSharedMutations(node) {
      if (isFunctionNode(node) && !isCalledFunction(node)) return;
      if (node.type === "AssignmentExpression") {
        reportAssignment(node);
      } else if (node.type === "UpdateExpression") {
        reportIfShared(node, mutationRootName(node.argument));
      } else if (node.type === "CallExpression") {
        reportIfShared(node, mutatingCallRootName(node));
      }
      for (const child of childNodes(node)) checkSharedMutations(child);
    }

    function collectViMockFactoryReferences(node) {
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
      for (const name of collectIdentifierReferences(factory.body)) {
        viMockFactoryReferences.add(name);
        markViMockCapturedMutable(name);
      }
    }

    function collectIdentifierReferences(node, names = new Set()) {
      if (!node) return names;
      if (node.type === "Identifier") names.add(node.name);
      for (const child of childNodes(node)) collectIdentifierReferences(child, names);
      return names;
    }
  },
);
