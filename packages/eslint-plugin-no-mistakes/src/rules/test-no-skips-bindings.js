"use strict";
const { resolveVariable } = require("./test-no-shared-state-aliases");
const { importSpecifierName } = require("./test-no-shared-state-callees");
const { unwrapExpression, isFunction } = require("./async-ast");
const NAMES = new Set(["it", "test", "describe"]);
const MODULES = new Set(["vitest", "@playwright/test"]);
const CHAIN = new Set(
  "skip skipIf runIf only todo fixme each for concurrent sequential fails fail serial parallel describe".split(
    " ",
  ),
);
function memberName(node) {
  return node.computed
    ? node.property.type === "Literal" && typeof node.property.value === "string"
      ? node.property.value
      : null
    : node.property.name;
}
function imported(node, context, namespace = false) {
  const variable = resolveVariable(node, context);
  for (const def of variable?.defs || []) {
    if (def.type !== "ImportBinding" || !MODULES.has(def.parent.source.value)) continue;
    if (namespace) {
      if (def.node.type === "ImportNamespaceSpecifier") return def.parent.source.value;
    } else {
      const name = importSpecifierName(def.node);
      if (NAMES.has(name)) return { framework: def.parent.source.value, name };
    }
  }
  // Unbound runner globals are considered only in files selected by lint config.
  if (!namespace && !variable?.defs.length && NAMES.has(node.name))
    return { framework: "vitest", name: node.name };
  return null;
}
function testBinding(node, context) {
  node = unwrapExpression(node);
  if (node.type === "Identifier") return imported(node, context);
  if (node.type === "CallExpression") return testBinding(node.callee, context);
  if (node.type === "TaggedTemplateExpression") return testBinding(node.tag, context);
  if (node.type !== "MemberExpression") return null;
  const name = memberName(node);
  if (NAMES.has(name) && node.object.type === "Identifier") {
    const framework = imported(node.object, context, true);
    if (framework) return { framework, name };
  }
  if (!CHAIN.has(name)) return null;
  const binding = testBinding(node.object, context);
  if (!binding) return null;
  if (name === "each" || name === "for") return { ...binding, table: name };
  return name === "describe" ? { ...binding, name } : binding;
}
function isContextSkip(node, context) {
  const object = unwrapExpression(node.object);
  if (object.type !== "Identifier") return false;
  const variable = resolveVariable(object, context);
  for (const def of variable?.defs || []) {
    const callback = def.node;
    if (def.type !== "Parameter" || !isFunction(callback)) continue;
    const call = callback.parent;
    if (call.type !== "CallExpression" || !call.arguments.includes(callback)) continue;
    const binding = testBinding(call.callee, context);
    if (binding?.framework !== "vitest" || binding.name === "describe" || binding.table === "each")
      continue;
    // each supplies table values; for preserves its second parameter as TestContext.
    const parameter = callback.params[binding.table === "for" ? 1 : 0];
    if (parameter?.type === "Identifier" && resolveVariable(parameter, context) === variable)
      return true;
  }
  return false;
}
module.exports = { memberName, testBinding, isContextSkip };
