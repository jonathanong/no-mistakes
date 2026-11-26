"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { UNANALYZABLE_SCHEMA } = require("./rule-unanalyzable");
const {
  importVisitors,
  recordTopLevelRequires,
  replayPendingCallbacks,
  reportOpaqueRegistration,
} = require("./test-no-shared-state-imports");

const {
  createRegistryReports,
  createViMockTracker,
  isInsideUncalledNestedFunction,
  walkSharedMutations,
} = require("./test-no-shared-state-analysis");

const {
  collectPatternNames,
  createCleanupTracker,
  firstNamedCallbackArgument,
  isMutableInitializer,
  isKnownTestCallee,
  isTestExtendCall,
  isTestCall,
  mutationRootName,
  namedCallbackArgument,
  registrationCall,
  setupCallbackKind,
} = require("./test-no-shared-state-helpers");
const { createMutationHandlers } = require("./test-no-shared-state-mutations");
const {
  createImportedTestAliases,
  recordImportedTests,
  recordTopLevelBindings,
} = require("./test-no-shared-state-aliases");
const { createRuleHelpers } = require("./test-no-shared-state-rule-helpers");

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow mutable module-scope test state", recommended: false },
    schema: [
      {
        type: "object",
        properties: {
          allowBeforeAllAssignments: { type: "boolean" },
          unanalyzable: UNANALYZABLE_SCHEMA,
        },
      },
    ],
    messages: {
      shared:
        "Shared mutable module-scope state between tests: use local variables inside each test instead.",
      unanalyzable: "Cannot tell whether {{subject}} is a test registration. Use a static import.",
    },
  },
  (context) => {
    const mutableTopLevel = new Set();
    const testCalleeNames = new Set(["it", "test", "describe"]);
    const namespaces = new Set();
    const testAliases = createImportedTestAliases(context);
    const pendingNamedCallbacks = [];
    const pendingNamedSetupCallbacks = [];
    const ruleOptions = context.options?.[0] ?? {};
    const cleanupTracker = createCleanupTracker(ruleOptions);
    const viMockTracker = createViMockTracker(context, mutableTopLevel);
    const suppressed = ruleSuppression(context, "test-no-shared-state");
    const registryReports = createRegistryReports(
      context,
      mutableTopLevel,
      cleanupTracker,
      (name) => viMockTracker.isCaptured(name),
      suppressed,
    );
    const depth = { test: 0, setup: 0 };
    const {
      calleeHasProperty,
      configureSerialMode,
      isDescribeCall,
      isInlineCallback,
      resolveFunctionCallback,
    } = createRuleHelpers(context, testCalleeNames, namespaces, testAliases);
    const mutationHandlers = createMutationHandlers({
      cleanupTracker,
      context,
      depths: { setup: () => depth.setup, test: () => depth.test },
      mutableTopLevel,
      registryReports,
      suppressed,
      viMockTracker,
    });

    function visitTopLevelVariableDeclaration(node) {
      recordTopLevelRequires(node, testCalleeNames, namespaces, testAliases, context);
      recordTopLevelBindings(node, {
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
      });
    }

    return {
      "Program > VariableDeclaration": visitTopLevelVariableDeclaration,
      "Program > ExportNamedDeclaration > VariableDeclaration": visitTopLevelVariableDeclaration,
      "Program:exit"() {
        replayPendingCallbacks({
          cleanupTracker,
          depth,
          mutationHandlers,
          pendingNamedCallbacks,
          pendingNamedSetupCallbacks,
          registryReports,
          walkSharedMutations,
        });
      },
      ...importVisitors(testCalleeNames, namespaces, testAliases, recordImportedTests),
      CallExpression(node) {
        viMockTracker.collectFactoryReferences(node);
        const registered = registrationCall(node);
        if (!registered) {
          reportOpaqueRegistration(node, context);
          return;
        }
        node = registered;
        const isShadowed = testAliases.isShadowed(node.callee);
        const isDescribe =
          !isShadowed && (isDescribeCall(node) || testAliases.isDescribeAliasCall(node.callee));
        const isSerialSuite =
          isDescribe &&
          (testAliases.hasProperty(node.callee, "serial") ||
            testAliases.isSerialAlias(node.callee));
        if (isDescribe) cleanupTracker.enterSuite(isSerialSuite);
        if (!isShadowed && configureSerialMode(node)) cleanupTracker.markCurrentSuiteSerial();
        const entered = !isShadowed && isTestCall(node, testCalleeNames, namespaces);
        if (!entered && !setupCallbackKind(node, testCalleeNames, namespaces))
          reportOpaqueRegistration(node, context);
        if (entered) {
          depth.test += 1;
          const callback = namedCallbackArgument(node.arguments);
          if (callback && !setupCallbackKind(node, testCalleeNames, namespaces))
            pendingNamedCallbacks.push({
              declaration: resolveFunctionCallback(node, callback),
              suiteKey: cleanupTracker.currentSuiteKey(),
            });
        }
        const setupKind = !isShadowed && setupCallbackKind(node, testCalleeNames, namespaces);
        const setupCallback = setupKind && firstNamedCallbackArgument(node.arguments);
        if (setupCallback) {
          pendingNamedSetupCallbacks.push({
            declaration: resolveFunctionCallback(node, setupCallback),
            suiteKey: cleanupTracker.currentSuiteKey(),
            kind: setupKind,
          });
        }
        if (setupKind) {
          depth.setup += 1;
          cleanupTracker.beginSetup(setupKind);
        }
      },
      AssignmentExpression(node) {
        if (isInsideUncalledNestedFunction(node, depth.test, depth.setup, isInlineCallback)) return;
        mutationHandlers.rememberAssignmentCleanup(node);
        mutationHandlers.reportAssignment(node);
      },
      UpdateExpression(node) {
        if (isInsideUncalledNestedFunction(node, depth.test, depth.setup, isInlineCallback)) return;
        mutationHandlers.reportIfShared(node, mutationRootName(node.argument));
      },
      "CallExpression:exit"(node) {
        const original = node;
        node = registrationCall(node);
        const isShadowed = !node || testAliases.isShadowed(node.callee);
        if (!isShadowed && isTestCall(node, testCalleeNames, namespaces)) depth.test -= 1;
        if (!isShadowed && setupCallbackKind(node, testCalleeNames, namespaces)) {
          depth.setup -= 1;
          cleanupTracker.endSetup();
        }
        const isInsideNested = isInsideUncalledNestedFunction(
          original,
          depth.test,
          depth.setup,
          isInlineCallback,
        );
        if (!isInsideNested) mutationHandlers.rememberCall(original);
        if (!isShadowed && (isDescribeCall(node) || testAliases.isDescribeAliasCall(node.callee))) {
          cleanupTracker.exitSuite();
        }
      },
    };
  },
);
