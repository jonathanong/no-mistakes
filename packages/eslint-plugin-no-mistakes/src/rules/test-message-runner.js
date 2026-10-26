"use strict";

const { resolveVariable } = require("./test-no-shared-state-aliases");
const { unwrap, memberName } = require("./test-message-expect");
const RUNNERS = new Set(["test", "it"]);
const MODULES = new Set(["vitest", "@jest/globals", "@playwright/test"]);

function namespace(node, context, seen) {
  node = unwrap(node);
  if (node?.type !== "Identifier") return false;
  const variable = resolveVariable(node, context);
  if (!variable || seen.has(variable)) return false;
  seen.add(variable);
  const def = variable.defs.length === 1 ? variable.defs[0] : null;
  if (def?.type === "ImportBinding")
    return (
      MODULES.has(def.parent?.source?.value) &&
      (def.node.type === "ImportNamespaceSpecifier" ||
        (def.node.type === "ImportDefaultSpecifier" &&
          def.parent.source.value === "@playwright/test"))
    );
  return (
    def?.type === "Variable" &&
    def.node?.parent?.kind === "const" &&
    def.node.id.type === "Identifier" &&
    !variable.references.some((ref) => ref.isWrite() && !ref.init) &&
    namespace(def.node.init, context, seen)
  );
}

function bindingRunner(node, context, seen = new Set()) {
  node = unwrap(node);
  if (node?.type === "CallExpression") return bindingRunner(node.callee, context, seen);
  if (node?.type === "MemberExpression") {
    const name = memberName(node);
    if (!name) return false;
    return (
      (RUNNERS.has(name) && namespace(node.object, context, new Set())) ||
      bindingRunner(node.object, context, seen)
    );
  }
  if (node?.type !== "Identifier") return false;
  const variable = resolveVariable(node, context);
  if (!variable || variable.defs.length === 0) return RUNNERS.has(node.name);
  if (seen.has(variable) || variable.defs.length !== 1) return false;
  seen.add(variable);
  const def = variable.defs[0];
  if (def.type === "Parameter") return false;
  if (def.type === "ImportBinding") {
    if (def.node.type === "ImportDefaultSpecifier")
      return def.parent.source.value === "@playwright/test";
    return (
      (MODULES.has(def.parent?.source?.value) &&
        RUNNERS.has(def.node.imported?.name || def.node.imported?.value)) ||
      RUNNERS.has(node.name)
    );
  }
  // Preserve module-level test/it fixtures while rejecting nested name reuse.
  const scope = variable.scope;
  if (
    RUNNERS.has(node.name) &&
    (["module", "global"].includes(scope?.type) ||
      (scope?.type === "function" && scope.block?.type === "Program"))
  )
    return true;
  if (
    def.type !== "Variable" ||
    def.node?.parent?.kind !== "const" ||
    !def.node.init ||
    variable.references.some((ref) => ref.isWrite() && !ref.init)
  )
    return false;
  if (def.node.id.type === "Identifier") return bindingRunner(def.node.init, context, seen);
  if (def.node.id.type !== "ObjectPattern" || !namespace(def.node.init, context, new Set()))
    return false;
  return def.node.id.properties.some(
    (property) =>
      property.type === "Property" &&
      property.value?.type === "Identifier" &&
      property.value.name === node.name &&
      RUNNERS.has(
        memberName({
          type: "MemberExpression",
          property: property.key,
          computed: property.computed,
        }),
      ),
  );
}

// Preserve the released conservative checks for calls rooted at test/it,
// even when the local binding cannot be proven to be a framework runner.
function legacyRunner(node) {
  node = unwrap(node);
  if (node?.type === "Identifier") return RUNNERS.has(node.name);
  if (node?.type === "MemberExpression")
    return Boolean(memberName(node)) && legacyRunner(node.object);
  if (node?.type === "CallExpression") return legacyRunner(node.callee);
  return false;
}

function isTestRunner(node, context) {
  return legacyRunner(node) || bindingRunner(node, context);
}

module.exports = { isTestRunner };
