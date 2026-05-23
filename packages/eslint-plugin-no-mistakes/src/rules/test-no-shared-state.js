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
  isFunctionNode,
  isMutableInitializer,
  isSetupCall,
  isTestCall,
  mutatingCallPropertyName,
  mutatingCallRootName,
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
    const functionDeclarations = new Map();
    const pendingNamedCallbacks = [];
    const pendingNamedSetupCallbacks = [];
    const cleanupTracker = createCleanupTracker();
    const viMockTracker = createViMockTracker(context, mutableTopLevel);
    const registryReports = createRegistryReports(
      context,
      mutableTopLevel,
      cleanupTracker,
      (name) => viMockTracker.isCaptured(name),
    );
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

    function rememberSetupCleanup(node, name) {
      if (
        name &&
        setupDepth > 0 &&
        mutableTopLevel.has(name) &&
        isModuleMutable(context, mutableTopLevel, node, name)
      ) {
        cleanupTracker.remember(name);
      }
    }

    function rememberSetupCallCleanup(node) {
      if (mutatingCallPropertyName(node) === "clear") {
        rememberSetupCleanup(node, mutatingCallRootName(node));
      }
    }

    function rememberAssignmentCleanup(node) {
      if (setupDepth === 0) return;
      if (!isResetAssignment(node, isMutableInitializer)) return;
      if (node.left.type === "Identifier") {
        rememberSetupCleanup(node, node.left.name);
        return;
      }
      rememberSetupCleanup(node, mutationRootName(node.left));
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
            viMockTracker.markIfCaptured(name);
          }
        }
      },
      "Program > FunctionDeclaration"(node) {
        if (node.id?.name) functionDeclarations.set(node.id.name, node);
      },
      "Program:exit"() {
        for (const { name, suiteKey, kind } of pendingNamedSetupCallbacks) {
          const declaration = functionDeclarations.get(name);
          if (!declaration) continue;
          const previousSetupDepth = setupDepth;
          setupDepth = 1;
          cleanupTracker.beginSetup(kind, suiteKey);
          checkSharedMutations(declaration.body);
          setupDepth = previousSetupDepth;
          cleanupTracker.endSetup();
        }
        testDepth = 1;
        for (const { name, suiteKey } of pendingNamedCallbacks) {
          const declaration = functionDeclarations.get(name);
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
              name: callback.name,
              suiteKey: cleanupTracker.currentSuiteKey(),
            });
          }
        }
        const setupKind = setupCallbackKind(node);
        if (setupKind) {
          const callback = namedCallbackArgument(node.arguments);
          if (callback) {
            pendingNamedSetupCallbacks.push({
              name: callback.name,
              suiteKey: cleanupTracker.currentSuiteKey(),
              kind: setupKind,
            });
          }
        }
        if (isSetupCall(node)) {
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
        if (isSetupCall(node)) {
          setupDepth -= 1;
          cleanupTracker.endSetup();
        }
        const isInsideNested = isInsideUncalledNestedFunction(node, testDepth, setupDepth);
        if (!isInsideNested) {
          rememberSetupCallCleanup(node);
          registryReports.remember(node, mutatingCallRootName(node), testDepth, setupDepth);
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
          rememberSetupCallCleanup(call);
          registryReports.remember(call, mutatingCallRootName(call), testDepth, setupDepth);
        },
        onUpdate: (update) => reportIfShared(update, mutationRootName(update.argument)),
      });
    }
  },
);
