"use strict";

const { unwrapExpression, unwrapSequence } = require("./async-ast");
const { requireSource } = require("./async-target-sources");
const { literalString } = require("./logical-call");
const { moduleLoad } = require("./module-load");
const { resolveVariable } = require("./test-no-shared-state-aliases");

const TEST_NAMES = new Set(["test", "it", "describe"]);

function loads() {
  return require("./no-vitest-sequential-loads");
}

function reassignedVitestRequire(callee, context) {
  const { bindingRoot, hasLaterWrite, staticName } = loads();
  const root = bindingRoot(callee);
  if (!root) return false;
  const variable = resolveVariable(root, context);
  if (!variable || variable.defs.length !== 1 || !hasLaterWrite(variable)) return false;
  const def = variable.defs[0];
  if (def.type !== "Variable" || def.node?.id?.type !== "Identifier") return false;
  const init = unwrapExpression(def.node.init);
  const loaded = init?.type === "MemberExpression" ? unwrapExpression(init.object) : init;
  const member =
    init?.type === "MemberExpression" ? staticName(init.property, init.computed) : null;
  if (init?.type === "MemberExpression" && member === null) return false;
  return requireSource(loaded, context) === "vitest" && (member === null || TEST_NAMES.has(member));
}

function reassignedDestructuredRunner(root, context) {
  const { hasLaterWrite, patternTests } = loads();
  if (root?.type !== "Identifier") return false;
  const variable = resolveVariable(root, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (
    def?.type !== "Variable" ||
    def.node?.id?.type !== "ObjectPattern" ||
    !hasLaterWrite(variable) ||
    requireSource(def.node.init, context) !== "vitest"
  ) {
    return false;
  }
  return patternTests(def.node.id).some(
    (entry) => !entry.dynamic && entry.local.name === root.name,
  );
}

function exportKey(pattern, name) {
  for (const property of pattern?.properties || []) {
    if (property.type !== "Property") continue;
    let value = property.value;
    while (value?.type === "AssignmentPattern") value = value.left;
    if (value?.type === "ObjectPattern") {
      const nested = exportKey(value, name);
      if (nested !== undefined) return nested;
    }
    if (value?.type !== "Identifier" || value.name !== name) continue;
    if (!property.computed && property.key?.type === "Identifier") return property.key.name;
    return literalString(property.key);
  }
  return undefined;
}

function constAlias(node, context, seen = new Set()) {
  let current = unwrapSequence(node);
  if (current?.type === "MemberExpression" || current?.type === "CallExpression") {
    let root = current;
    while (root?.type === "MemberExpression" || root?.type === "CallExpression") {
      root = unwrapSequence(root.type === "CallExpression" ? root.callee : root.object);
    }
    if (root?.type !== "Identifier") return current;
    const followed = constAlias(root, context, seen);
    return followed === root ? current : followed;
  }
  while (current?.type === "Identifier") {
    if (seen.has(current.name)) return current;
    const variable = resolveVariable(current, context);
    const def = variable?.defs.length === 1 ? variable.defs[0] : null;
    if (
      def?.type !== "Variable" ||
      def.node?.parent?.kind !== "const" ||
      def.node.id?.type !== "Identifier" ||
      !def.node.init
    ) {
      return current;
    }
    seen.add(current.name);
    const next = unwrapSequence(def.node.init);
    if (next?.type !== "Identifier") return next;
    current = next;
  }
  return current;
}

function unknownComputedExport(root, context) {
  if (root?.type !== "Identifier") return false;
  const variable = resolveVariable(root, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (def?.type !== "Variable" || def.node?.id?.type !== "ObjectPattern") return false;
  const loaded = moduleLoad(def.node.init, context);
  if (loaded?.source !== "vitest" || loaded.exportName !== "*") return false;
  return exportKey(def.node.id, root.name) === null;
}

function rejectedRunner(root, context) {
  return (
    reassignedDestructuredRunner(root, context) ||
    renamedNonRunner(root, context) ||
    reassignedVitestRequire(root, context) ||
    unknownComputedExport(root, context)
  );
}

function renamedNonRunner(root, context) {
  if (root?.type !== "Identifier") return false;
  const variable = resolveVariable(root, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (def?.type !== "Variable" || def.node?.id?.type !== "ObjectPattern") return false;
  const loaded = moduleLoad(def.node.init, context);
  if (loaded?.source !== "vitest" || loaded.exportName !== "*") return false;
  const exported = exportKey(def.node.id, root.name);
  return typeof exported === "string" && !TEST_NAMES.has(exported);
}

module.exports = {
  constAlias,
  reassignedDestructuredRunner,
  reassignedVitestRequire,
  rejectedRunner,
  renamedNonRunner,
};
