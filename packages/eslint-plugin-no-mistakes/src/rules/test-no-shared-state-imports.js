"use strict";

const { unwrapExpression } = require("./async-ast");
const { isReassigned } = require("./async-target-bindings");
const { requireResolution, requireSource } = require("./async-target-sources");
const { literalString } = require("./logical-call");
const { isUnresolved, moduleLoad } = require("./module-load");
const { reportUnanalyzable } = require("./rule-unanalyzable");
const { hidesComputedKey, resolveVariable } = require("./test-no-shared-state-aliases");

const TEST_NAMES = new Set(["describe", "it", "test"]);
const REGISTRATION_MODIFIERS = new Set(["concurrent", "each", "fails", "only", "skip", "todo"]);

function patternNames(pattern) {
  const found = [];
  for (const property of pattern?.properties || []) {
    if (property.type !== "Property" || property.computed) continue;
    let value = property.value;
    while (value?.type === "AssignmentPattern") value = value.left;
    if (value?.type !== "Identifier") continue;
    const key =
      property.key?.type === "Identifier" ? property.key.name : literalString(property.key);
    if (TEST_NAMES.has(key)) found.push({ key, local: value });
  }
  return found;
}

function recordModuleBinding(source, id, testCalleeNames, namespaces, testAliases, context) {
  if ((source !== "vitest" && source !== "@playwright/test") || !id) return;
  if (id.type === "Identifier") {
    if (context && isReassigned(id, context)) return;
    (source === "vitest" ? namespaces : testCalleeNames).add(id.name);
    testAliases.add(id, source === "vitest" ? "namespace" : "test");
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
  if (init?.type !== "MemberExpression" || init.computed) return false;
  if (declaration?.id?.type !== "Identifier") return false;
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
  if (node?.importKind === "type" || node.moduleReference?.type !== "TSExternalModuleReference")
    return;
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
    if (
      declaration.parent?.kind !== "const" ||
      declaration.id?.type !== "ObjectPattern" ||
      !isUnresolved(moduleLoad(declaration.init, context))
    ) {
      continue;
    }
    for (const { local } of patternNames(declaration.id)) testCalleeNames.delete(local.name);
  }
}

function dynamicBinding(callee, context, seen = new Set()) {
  if (callee?.type !== "Identifier" || seen.has(callee.name)) return null;
  seen.add(callee.name);
  const variable = resolveVariable(callee, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (def?.type !== "Variable" || isReassigned(callee, context)) return null;
  const init = unwrapExpression(def.node.init);
  const alias =
    def.parent?.kind === "const" &&
    def.node.id?.type === "Identifier" &&
    init?.type === "Identifier";
  return alias ? dynamicBinding(init, context, seen) : { def, name: callee.name };
}

function dynamicDestructuredTest(callee, context) {
  const found = dynamicBinding(callee, context);
  const loaded = moduleLoad(found?.def.node?.init, context);
  if (found?.def.node?.id?.type !== "ObjectPattern" || !isUnresolved(loaded)) return false;
  return patternNames(found.def.node.id).some((entry) => entry.local.name === found.name);
}

function dynamicMemberTest(callee, context) {
  const declaration = dynamicBinding(callee, context)?.def.node;
  const init = unwrapExpression(declaration?.init);
  if (declaration?.id?.type !== "Identifier") return false;
  if (init?.type !== "MemberExpression" || init.computed) return false;
  const key = init.property?.type === "Identifier" ? init.property.name : null;
  return TEST_NAMES.has(key) && isUnresolved(moduleLoad(init.object, context));
}

function dynamicLetReceiver(node, context) {
  const target = unwrapExpression(node);
  if (target?.type !== "Identifier") return false;
  const variable = resolveVariable(target, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (def?.type !== "Variable" || def.parent?.kind === "const") return false;
  if (requireResolution(def.node?.init, context)?.source !== null) return false;
  const id = def.node.id;
  if (isReassigned(target, context)) return false;
  if (id?.type === "Identifier") return true;
  return (id?.properties || []).some(
    (property) => !property.computed && property.value?.name === target.name,
  );
}

function registrationParts(node) {
  let current = unwrapExpression(node);
  while (current?.type === "CallExpression") current = unwrapExpression(current.callee);
  let member = null;
  while (current?.type === "MemberExpression" && !current.computed) {
    const name = current.property?.name;
    if (TEST_NAMES.has(name)) {
      member = current;
      break;
    }
    if (!REGISTRATION_MODIFIERS.has(name)) break;
    current = unwrapExpression(current.object);
    while (current?.type === "CallExpression") current = unwrapExpression(current.callee);
  }
  return { alias: current, member };
}

function reportOpaqueRegistration(call, context, suppressed, source) {
  const callee = unwrapExpression(call?.callee);
  // `each(rows)` builds the registration that the following call invokes.
  if (callee?.type === "MemberExpression" && !callee.computed && callee.property?.name === "each") {
    return;
  }
  const { alias, member: registration } = registrationParts(callee);
  const receiver = unwrapExpression(registration?.object);
  const unresolvedMember =
    registration &&
    !hidesComputedKey(receiver, context) &&
    (isUnresolved(moduleLoad(receiver, context)) || dynamicLetReceiver(receiver, context));
  if (
    !unresolvedMember &&
    !dynamicDestructuredTest(alias, context) &&
    !dynamicMemberTest(alias, context)
  ) {
    return;
  }
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
