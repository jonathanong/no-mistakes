"use strict";

const { rule } = require("../helpers");
const {
  createRegistryReports,
  createViMockTracker,
  isInsideUncalledNestedFunction,
  isModuleMutable,
  isResetAssignment,
  walkSharedMutations,
} = require("./test-no-shared-state-analysis");
const {
  calleeName,
  collectPatternNames,
  createCleanupTracker,
  firstNamedCallbackArgument,
  isFunctionNode,
  isMutableInitializer,
  isTestCall,
  mutatingCallPropertyName,
  mutatingCallTarget,
  mutationPath,
  mutationRootName,
  namedCallbackArgument,
  setupCallbackKind,
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
    const pendingNamedCallbacks = [];
    const pendingNamedSetupCallbacks = [];
    const cleanupTracker = createCleanupTracker();
    const viMockTracker = createViMockTracker(context, mutableTopLevel);
    const registryReports = createRegistryReports({
      context,
      mutableTopLevel,
      cleanupTracker,
      isCaptured: (name) => viMockTracker.isCaptured(name),
    });
    let testDepth = 0;
    let setupDepth = 0;

    function reportIfShared(node, name) {
      if (
        name &&
        testDepth > 0 &&
        setupDepth === 0 &&
        !viMockTracker.isCaptured(name) &&
        isModuleMutable(context, mutableTopLevel, node, name)
      ) {
        context.report({ node, messageId: "shared" });
      }
    }

    function reportAssignment(node) {
      for (const name of collectPatternNames(node.left)) reportIfShared(node, name);
      if (node.left.type !== "MemberExpression") return;
      reportIfShared(node, mutationRootName(node.left));
    }

    function rememberSetupCleanup(node, name, path) {
      if (
        name &&
        setupDepth > 0 &&
        mutableTopLevel.has(name) &&
        isModuleMutable(context, mutableTopLevel, node, name)
      ) {
        cleanupTracker.remember(path);
      }
    }

    function rememberCall(node) {
      const { name, path } = mutatingCallTarget(node);
      if (mutatingCallPropertyName(node) === "clear") {
        rememberSetupCleanup(node, name, path);
      }
      registryReports.remember(node, name, path, testDepth, setupDepth);
    }

    function rememberAssignmentCleanup(node) {
      if (setupDepth === 0) return;
      if (!isResetAssignment(node, isMutableInitializer)) return;
      if (node.left.type === "Identifier") {
        rememberSetupCleanup(node, node.left.name, node.left.name);
        return;
      }
      rememberSetupCleanup(node, mutationRootName(node.left), mutationPath(node.left.object));
    }

    function resolveFunctionCallback(node, callback) {
      let scope = context.sourceCode.getScope(node);
      while (scope) {
        const variable = scope.variables.find((candidate) => candidate.name === callback.name);
        const declaration = variable?.defs[0]?.node;
        if (declaration?.type === "FunctionDeclaration") return declaration;
        if (declaration?.type === "VariableDeclarator" && isFunctionNode(declaration.init)) {
          return declaration.init;
        }
        scope = scope.upper;
      }
      return null;
    }

    return {
      "Program > VariableDeclaration"(node) {
        for (const declaration of node.declarations) {
          if (node.kind === "const" && !isMutableInitializer(declaration.init)) continue;
          for (const name of collectPatternNames(declaration.id)) {
            mutableTopLevel.add(name);
            viMockTracker.markIfCaptured(name);
          }
        }
      },
      "Program:exit"() {
        for (const { declaration, suiteKey, kind } of pendingNamedSetupCallbacks) {
          if (!declaration) continue;
          const previousSetupDepth = setupDepth;
          setupDepth = 1;
          cleanupTracker.beginSetup(kind, suiteKey);
          checkSharedMutations(declaration.body);
          setupDepth = previousSetupDepth;
          cleanupTracker.endSetup();
        }
        testDepth = 1;
        for (const { declaration, suiteKey } of pendingNamedCallbacks) {
          if (!declaration) continue;
          cleanupTracker.setReplaySuite(suiteKey);
          checkSharedMutations(declaration.body);
          cleanupTracker.clearReplaySuite();
        }
        registryReports.flush();
      },
      CallExpression(node) {
        viMockTracker.collectFactoryReferences(node);
        if (calleeName(node.callee) === "describe") cleanupTracker.enterSuite();
        if (isTestCall(node)) {
          testDepth += 1;
          const callback = namedCallbackArgument(node.arguments);
          if (callback) {
            pendingNamedCallbacks.push({
              declaration: resolveFunctionCallback(node, callback),
              suiteKey: cleanupTracker.currentSuiteKey(),
            });
          }
        }
        const setupKind = setupCallbackKind(node);
        if (setupKind) {
          const callback = firstNamedCallbackArgument(node.arguments);
          if (callback) {
            pendingNamedSetupCallbacks.push({
              declaration: resolveFunctionCallback(node, callback),
              suiteKey: cleanupTracker.currentSuiteKey(),
              kind: setupKind,
            });
          }
        }
        if (setupKind) {
          setupDepth += 1;
          cleanupTracker.beginSetup(setupKind);
        }
      },
      AssignmentExpression(node) {
        if (isInsideUncalledNestedFunction(node, testDepth, setupDepth)) return;
        rememberAssignmentCleanup(node);
        reportAssignment(node);
      },
      UpdateExpression(node) {
        if (isInsideUncalledNestedFunction(node, testDepth, setupDepth)) return;
        reportIfShared(node, mutationRootName(node.argument));
      },
      "CallExpression:exit"(node) {
        if (isTestCall(node)) testDepth -= 1;
        if (setupCallbackKind(node)) {
          setupDepth -= 1;
          cleanupTracker.endSetup();
        }
        const isInsideNested = isInsideUncalledNestedFunction(node, testDepth, setupDepth);
        if (!isInsideNested) {
          rememberCall(node);
        }
        if (calleeName(node.callee) === "describe") cleanupTracker.exitSuite();
      },
    };

    function checkSharedMutations(node) {
      walkSharedMutations(node, {
        onAssignment: (assignment) => {
          rememberAssignmentCleanup(assignment);
          reportAssignment(assignment);
        },
        onCall: (call) => {
          rememberCall(call);
        },
        onUpdate: (update) => reportIfShared(update, mutationRootName(update.argument)),
      });
    }
  },
);
