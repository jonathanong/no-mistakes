"use strict";
const { resolveVariable } = require("./test-no-shared-state-aliases");
const { importSpecifierName } = require("./test-no-shared-state-callees");
const { memberName, testBinding } = require("./test-no-skips-bindings");
const { unwrapExpression } = require("./async-ast");
const HOOKS = new Set(["beforeAll", "beforeEach", "afterAll", "afterEach"]);
const CONFIG = new Set(["defineConfig", "defineProject", "mergeConfig"]);
function bindingName(node, context, modules, names) {
  node = unwrapExpression(node);
  if (node.type === "MemberExpression" && node.object.type === "Identifier") {
    const variable = resolveVariable(node.object, context);
    const name = memberName(node);
    return names.has(name) &&
      variable?.defs.some(
        (def) =>
          def.type === "ImportBinding" &&
          modules.has(def.parent.source.value) &&
          def.node.type === "ImportNamespaceSpecifier",
      )
      ? name
      : null;
  }
  if (node.type !== "Identifier") return null;
  const variable = resolveVariable(node, context);
  for (const def of variable?.defs || []) {
    if (def.type === "ImportBinding" && modules.has(def.parent.source.value)) {
      const name = importSpecifierName(def.node);
      if (names.has(name)) return name;
    }
  }
  if (modules.has("vitest") && !variable?.defs.length && names.has(node.name)) return node.name;
  return null;
}
function configCall(node, context) {
  return bindingName(node.callee, context, new Set(["vitest/config", "vite"]), CONFIG);
}
function overrideCall(node, context) {
  const callee = unwrapExpression(node.callee);
  if (
    callee.type === "MemberExpression" &&
    memberName(callee) === "setConfig" &&
    bindingName(callee.object, context, new Set(["vitest"]), new Set(["vi"]))
  )
    return "runtime";
  if (bindingName(callee, context, new Set(["vitest"]), HOOKS)) return "hook";
  const test = testBinding(callee, context);
  if (test?.framework !== "vitest") return null;
  if (
    callee.type === "MemberExpression" &&
    new Set(["each", "for", "skipIf", "runIf"]).has(memberName(callee))
  )
    return null;
  return "test";
}
module.exports = { configCall, overrideCall };
