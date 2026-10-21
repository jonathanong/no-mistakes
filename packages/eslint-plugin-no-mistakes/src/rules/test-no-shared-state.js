"use strict";

const { rule } = require("../helpers");

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
  setupCallbackKind,
} = require("./test-no-shared-state-helpers");
const { createMutationHandlers } = require("./test-no-shared-state-mutations");
const {
  createImportedTestAliases,
  isDescribeBinding,
  recordImportedTests,
} = require("./test-no-shared-state-aliases");
const { createRuleHelpers } = require("./test-no-shared-state-rule-helpers");

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow mutable module-scope test state", recommended: false },
    schema: [{ type: "object", properties: { allowBeforeAllAssignments: { type: "boolean" } } }],
    messages: {
      shared:
        "Shared mutable module-scope state between tests: use local variables inside each test instead.",
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
    const registryReports = createRegistryReports(
      context,
      mutableTopLevel,
      cleanupTracker,
      (name) => viMockTracker.isCaptured(name),
    );
    let testDepth = 0;
    let setupDepth = 0;
    const {
      calleeHasProperty,
      configureSerialMode,
      isDescribeCall,
      isInlineCallback,
      resolveFunctionCallback,
    } = createRuleHelpers(context, testCalleeNames, namespaces);
    const mutationHandlers = createMutationHandlers({
      cleanupTracker,
      context,
      depths: { setup: () => setupDepth, test: () => testDepth },
      mutableTopLevel,
      registryReports,
      viMockTracker,
    });

    function visitTopLevelVariableDeclaration(node) {
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
            testAliases.add(declaration.id, "describe");
          }
        }
        if (node.kind === "const" && !isMutableInitializer(declaration.init)) continue;
        for (const name of collectPatternNames(declaration.id)) {
          mutableTopLevel.add(name);
          viMockTracker.markIfCaptured(name);
        }
      }
    }

    return {
      "Program > VariableDeclaration": visitTopLevelVariableDeclaration,
      "Program > ExportNamedDeclaration > VariableDeclaration": visitTopLevelVariableDeclaration,
      "Program:exit"() {
        for (const { declaration, suiteKey, kind } of pendingNamedSetupCallbacks) {
          if (!declaration) continue;
          const previousSetupDepth = setupDepth;
          setupDepth = 1;
          cleanupTracker.beginSetup(kind, suiteKey);
          walkSharedMutations(declaration.body, mutationHandlers.mutationWalk);
          setupDepth = previousSetupDepth;
          cleanupTracker.endSetup();
        }
        testDepth = 1;
        for (const { declaration, suiteKey } of pendingNamedCallbacks) {
          if (!declaration) continue;
          cleanupTracker.setReplaySuite(suiteKey);
          walkSharedMutations(declaration.body, mutationHandlers.mutationWalk);
          cleanupTracker.clearReplaySuite();
        }
        registryReports.flush();
      },
      ImportDeclaration(node) {
        recordImportedTests(node, testCalleeNames, namespaces, testAliases);
      },
      CallExpression(node) {
        viMockTracker.collectFactoryReferences(node);
        const isShadowed = testAliases.isShadowed(node.callee);
        const isDescribe =
          !isShadowed && (isDescribeCall(node) || testAliases.isDescribeAliasCall(node.callee));
        const isSerialSuite = isDescribe && testAliases.hasProperty(node.callee, "serial");
        if (isDescribe) cleanupTracker.enterSuite(isSerialSuite);
        if (configureSerialMode(node)) cleanupTracker.markCurrentSuiteSerial();
        if (!isShadowed && isTestCall(node, testCalleeNames, namespaces)) {
          testDepth += 1;
          const callback = namedCallbackArgument(node.arguments);
          if (callback && !setupCallbackKind(node, testCalleeNames, namespaces)) {
            pendingNamedCallbacks.push({
              declaration: resolveFunctionCallback(node, callback),
              suiteKey: cleanupTracker.currentSuiteKey(),
            });
          }
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
          setupDepth += 1;
          cleanupTracker.beginSetup(setupKind);
        }
      },
      AssignmentExpression(node) {
        if (isInsideUncalledNestedFunction(node, testDepth, setupDepth, isInlineCallback)) return;
        mutationHandlers.rememberAssignmentCleanup(node);
        mutationHandlers.reportAssignment(node);
      },
      UpdateExpression(node) {
        if (isInsideUncalledNestedFunction(node, testDepth, setupDepth, isInlineCallback)) return;
        mutationHandlers.reportIfShared(node, mutationRootName(node.argument));
      },
      "CallExpression:exit"(node) {
        const isShadowed = testAliases.isShadowed(node.callee);
        if (!isShadowed && isTestCall(node, testCalleeNames, namespaces)) testDepth -= 1;
        if (!isShadowed && setupCallbackKind(node, testCalleeNames, namespaces)) {
          setupDepth -= 1;
          cleanupTracker.endSetup();
        }
        const isInsideNested = isInsideUncalledNestedFunction(
          node,
          testDepth,
          setupDepth,
          isInlineCallback,
        );
        if (!isInsideNested) mutationHandlers.rememberCall(node);
        if (!isShadowed && (isDescribeCall(node) || testAliases.isDescribeAliasCall(node.callee))) {
          cleanupTracker.exitSuite();
        }
      },
    };
  },
);
