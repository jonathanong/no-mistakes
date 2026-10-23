"use strict";
const { options } = require("../helpers");
const { resolveVariable, stableConstInitializer } = require("./async-target-bindings");
const { importSpecifierName } = require("./test-no-shared-state-callees");
const { literalString } = require("./module-mock-helpers");
const { unwrapExpression, isFunction } = require("./async-ast");
const NAMES = new Set(["it", "test", "describe"]);
const MODULES = new Set(["vitest", "@playwright/test"]);
const CONFIGURED_NAMESPACE = "configured-test-module";
const testModulesByContext = new WeakMap();
// extend is a transparent Playwright/Vitest fixture wrapper, including chained .extend().
const CHAIN = new Set(
  "skip skipIf runIf only todo fixme each for concurrent sequential fails fail serial parallel describe extend".split(
    " ",
  ),
);
function admit(source, importedName, namespace, testModules) {
  if (MODULES.has(source)) {
    if (namespace) return importedName === "*" ? source : null;
    return NAMES.has(importedName) ? { framework: source, name: importedName } : null;
  }
  if (!testModules.has(source) || importedName !== (namespace ? "*" : "test")) return null;
  return namespace ? CONFIGURED_NAMESPACE : { framework: "@playwright/test", name: "test" };
}
function immutableKey(node, context) {
  const seen = new Set();
  while (node) {
    node = unwrapExpression(node);
    const literal = literalString(node);
    if (literal !== null) return literal;
    node = stableConstInitializer(node, context, seen);
  }
  return null;
}
function memberName(node, context) {
  if (!node.computed && node.property.type === "Identifier") return node.property.name;
  return immutableKey(node.property, context);
}
function unwrapBinding(node) {
  let current = node;
  while (current.type === "AssignmentPattern") current = current.left;
  return current;
}
function propertyKey(property) {
  if (!property.computed && property.key.type === "Identifier") return property.key.name;
  return literalString(property.key);
}
// The key that introduces the identifier, after defaults and nested objects.
// `const { test: t = x } = require("vitest")` and
// `const { runner: { test: t } = fallback } = require("vitest")` both yield `test`.
function patternBindingName(pattern, name) {
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const value = unwrapBinding(property.value);
    if (value.type === "Identifier" && value.name === name) return propertyKey(property);
    if (value.type === "ObjectPattern") {
      const nested = patternBindingName(value, name);
      if (nested) return nested;
    }
  }
  return null;
}
function testModulesFor(context) {
  const cached = testModulesByContext.get(context);
  if (cached) return cached;
  const modules = new Set(options(context).testModules || []);
  testModulesByContext.set(context, modules);
  return modules;
}

function imported(node, context, namespace = false, seen = new Set()) {
  if (!node || seen.has(node)) return null;
  seen.add(node);
  const testModules = testModulesFor(context);
  const variable = resolveVariable(node, context);
  for (const def of variable?.defs || []) {
    let source = null;
    let importedName = null;
    if (def.type === "ImportBinding") {
      source = def.parent.source.value;
      if (def.node.type === "ImportNamespaceSpecifier") importedName = "*";
      else if (def.node.type === "ImportDefaultSpecifier")
        importedName = source === "@playwright/test" ? "test" : null;
      else importedName = importSpecifierName(def.node);
    } else if (def.type === "Variable") {
      const declaration = def.node;
      if (declaration.parent.kind !== "const") continue;
      let init = unwrapExpression(declaration.init || {});
      const awaited = init.type === "AwaitExpression";
      if (awaited) init = unwrapExpression(init.argument);
      if (
        init.type === "CallExpression" &&
        init.callee.type === "Identifier" &&
        init.callee.name === "require"
      ) {
        const requireVariable = resolveVariable(init.callee, context);
        if (requireVariable?.defs.length) continue;
        source = init.arguments[0]?.type === "Literal" ? init.arguments[0].value : null;
      } else if (init.type === "ImportExpression" && awaited) {
        source = init.source?.type === "Literal" ? init.source.value : null;
      }
      if (declaration.id.type === "ObjectPattern") {
        importedName = patternBindingName(declaration.id, node.name);
      } else if (declaration.id.type === "Identifier" && declaration.id.name === node.name) {
        importedName = "*";
      }
      if (!source && init.type === "Identifier" && declaration.id.type === "ObjectPattern") {
        const namespaceSource = imported(init, context, true, seen);
        if (
          (namespaceSource === "vitest" || namespaceSource === "@playwright/test") &&
          NAMES.has(importedName)
        ) {
          return { framework: namespaceSource, name: importedName };
        }
      }
    }
    const admitted = admit(source, importedName, namespace, testModules);
    if (admitted) return admitted;
  }
  // Unbound runner globals are considered only in files selected by lint config.
  if (!namespace && !variable?.defs.length && NAMES.has(node.name))
    return { framework: "vitest", name: node.name };
  return null;
}
function testBinding(node, context, visited = new Set()) {
  if (!node?.type) return null;
  node = unwrapExpression(node);
  if (!node?.type) return null;
  if (node.type === "Identifier") {
    const binding = imported(node, context);
    if (binding) return binding;
    const variable = resolveVariable(node, context);
    if (!variable || visited.has(variable) || variable.defs.length !== 1) return null;
    visited.add(variable);
    const definition = variable.defs[0];
    if (definition.type !== "Variable" || definition.node.parent.kind !== "const") return null;
    return testBinding(definition.node.init, context, visited);
  }
  if (node.type === "CallExpression") return testBinding(node.callee, context, visited);
  if (node.type === "TaggedTemplateExpression") return testBinding(node.tag, context, visited);
  if (node.type !== "MemberExpression") return null;
  const name = memberName(node, context);
  if (NAMES.has(name) && node.object.type === "Identifier") {
    const framework = imported(node.object, context, true);
    if (framework && (framework !== CONFIGURED_NAMESPACE || name === "test"))
      return {
        framework: framework === CONFIGURED_NAMESPACE ? "@playwright/test" : framework,
        name,
      };
  }
  if (!CHAIN.has(name)) return null;
  const binding = testBinding(node.object, context, visited);
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
