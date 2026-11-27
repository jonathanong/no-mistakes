"use strict";

const { unwrapExpression } = require("./async-ast");
const { isReassigned } = require("./async-target-bindings");
const { requireResolution, requireSource } = require("./async-target-sources");
const { literalString } = require("./logical-call");
const { isUnresolved, moduleLoad } = require("./module-load");
const { reportUnanalyzable } = require("./rule-unanalyzable");
const { resolveVariable } = require("./test-no-shared-state-aliases");

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

function recordModuleBinding(source, id, testCalleeNames, namespaces, testAliases, context) {
  if ((source !== "vitest" && source !== "@playwright/test") || !id) return;
  if (id.type === "Identifier") {
    if (context && isReassigned(id, context)) return;
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
    if (context && isReassigned(local, context)) continue;
    testCalleeNames.add(local.name);
    testAliases.add(local, key);
  }
}

function recordMemberRequire(declaration, testCalleeNames, testAliases, context) {
  const init = unwrapExpression(declaration?.init);
  if (
    init?.type !== "MemberExpression" ||
    init.computed ||
    declaration?.id?.type !== "Identifier"
  ) {
    return false;
  }
  const source = requireSource(init.object, context);
  const key = init.property?.type === "Identifier" ? init.property.name : null;
  if ((source !== "vitest" && source !== "@playwright/test") || !TEST_NAMES.has(key)) return false;
  if (!isReassigned(declaration.id, context)) {
    testCalleeNames.add(declaration.id.name);
    testAliases.add(declaration.id, key);
  }
  return true;
}

function recordRequireDeclarator(declaration, testCalleeNames, namespaces, testAliases, context) {
  if (recordMemberRequire(declaration, testCalleeNames, testAliases, context)) return;
  recordModuleBinding(
    requireSource(declaration?.init, context),
    declaration?.id,
    testCalleeNames,
    namespaces,
    testAliases,
    context,
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

function dynamicDestructuredTest(callee, context) {
  if (callee?.type !== "Identifier") return false;
  const variable = resolveVariable(callee, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (def?.type !== "Variable" || def.node?.id?.type !== "ObjectPattern") return false;
  const resolution = requireResolution(def.node.init, context);
  if (!resolution || resolution.source !== null) return false;
  return patternNames(def.node.id).some((entry) => entry.local.name === callee.name);
}

function reportOpaqueRegistration(call, context, suppressed, source) {
  const callee = unwrapExpression(call?.callee);
  const member =
    callee?.type === "MemberExpression" && !callee.computed ? callee.property?.name : null;
  const unresolvedMember =
    (member === "test" || member === "it" || member === "describe") &&
    isUnresolved(moduleLoad(callee.object, context));
  if (!unresolvedMember && !dynamicDestructuredTest(callee, context)) return;
  if (!suppressed(source)) {
    reportUnanalyzable(context, call.loc ? call : source, "this test registration");
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
