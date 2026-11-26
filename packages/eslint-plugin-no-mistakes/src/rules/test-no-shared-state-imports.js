"use strict";

const { unwrapExpression } = require("./async-ast");
const { requireSource } = require("./async-target-sources");
const { literalString } = require("./logical-call");
const { isUnresolved, moduleLoad } = require("./module-load");
const { reportUnanalyzable } = require("./rule-unanalyzable");

const TEST_NAMES = new Set(["describe", "it", "test"]);

function patternNames(pattern) {
  const found = [];
  for (const property of pattern?.properties || []) {
    if (property.type !== "Property") continue;
    let value = property.value;
    while (value?.type === "AssignmentPattern") value = value.left;
    if (value?.type !== "Identifier") continue;
    const key = property.computed
      ? literalString(property.key)
      : property.key?.type === "Identifier"
        ? property.key.name
        : literalString(property.key);
    if (TEST_NAMES.has(key)) found.push({ key, local: value });
  }
  return found;
}

function recordModuleBinding(source, id, testCalleeNames, namespaces, testAliases) {
  if ((source !== "vitest" && source !== "@playwright/test") || !id) return;
  if (id.type === "Identifier") {
    if (source === "vitest") {
      namespaces.add(id.name);
      testAliases.add(id, "namespace");
    } else {
      testCalleeNames.add(id.name);
      testAliases.add(id, "test");
    }
    return;
  }
  for (const { key, local } of patternNames(id)) {
    testCalleeNames.add(local.name);
    testAliases.add(local, key);
  }
}

function recordRequireDeclarator(declaration, testCalleeNames, namespaces, testAliases, context) {
  recordModuleBinding(
    requireSource(declaration?.init, context),
    declaration?.id,
    testCalleeNames,
    namespaces,
    testAliases,
  );
}

function recordImportEquals(node, testCalleeNames, namespaces, testAliases) {
  if (node?.importKind === "type" || node.moduleReference?.type !== "TSExternalModuleReference") {
    return;
  }
  recordModuleBinding(
    literalString(node.moduleReference.expression),
    node.id,
    testCalleeNames,
    namespaces,
    testAliases,
  );
}

function recordTopLevelRequires(node, testCalleeNames, namespaces, testAliases, context) {
  for (const declaration of node.declarations) {
    recordRequireDeclarator(declaration, testCalleeNames, namespaces, testAliases, context);
  }
}

function reportOpaqueRegistration(call, context) {
  const callee = unwrapExpression(call?.callee);
  if (callee?.type !== "MemberExpression" || callee.computed) return;
  const name = callee.property?.name;
  if (name !== "test" && name !== "it" && name !== "describe") return;
  if (isUnresolved(moduleLoad(callee.object, context))) {
    reportUnanalyzable(context, call, "this test registration");
  }
}

function replayPendingCallbacks(state) {
  const { depth, cleanupTracker, mutationHandlers, registryReports, walkSharedMutations } = state;
  for (const { declaration, suiteKey, kind } of state.pendingNamedSetupCallbacks) {
    if (!declaration) continue;
    const previousSetupDepth = depth.setup;
    depth.setup = 1;
    cleanupTracker.beginSetup(kind, suiteKey);
    walkSharedMutations(declaration.body, mutationHandlers.mutationWalk);
    depth.setup = previousSetupDepth;
    cleanupTracker.endSetup();
  }
  depth.test = 1;
  for (const { declaration, suiteKey } of state.pendingNamedCallbacks) {
    if (!declaration) continue;
    cleanupTracker.setReplaySuite(suiteKey);
    walkSharedMutations(declaration.body, mutationHandlers.mutationWalk);
    cleanupTracker.clearReplaySuite();
  }
  registryReports.flush();
}

function importVisitors(testCalleeNames, namespaces, testAliases, recordImportedTests) {
  return {
    ImportDeclaration(node) {
      recordImportedTests(node, testCalleeNames, namespaces, testAliases);
    },
    TSImportEqualsDeclaration(node) {
      recordImportEquals(node, testCalleeNames, namespaces, testAliases);
    },
  };
}

module.exports = {
  importVisitors,
  recordTopLevelRequires,
  replayPendingCallbacks,
  reportOpaqueRegistration,
};
