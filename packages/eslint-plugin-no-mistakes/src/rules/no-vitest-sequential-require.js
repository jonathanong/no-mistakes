"use strict";

const { unwrapExpression } = require("./async-ast");
const { requireSource } = require("./async-target-sources");
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

module.exports = { reassignedDestructuredRunner, reassignedVitestRequire };
