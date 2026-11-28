"use strict";

const { unwrapExpression } = require("./async-ast");
const { isReassigned, memberPropertyName, resolveVariable } = require("./async-target-bindings");
const { importSpecifierName, requireSource } = require("./async-target-sources");
const { literalString } = require("./logical-call");

const NODE_MODULES = new Set(["fs", "fs/promises", "path", "os", "url", "util", "child_process"]);
const BUILTIN_EXTERNAL = new Set(["fs-extra", "yaml", "js-yaml", "vitest", "@jest/globals"]);
const FRAMEWORK_GLOBALS = new Map([
  ["vi", "vitest.vi"],
  ["jest", "@jest/globals.jest"],
]);
const FRAMEWORK_MOCKS = new Set(["vitest.vi.mock", "@jest/globals.jest.mock"]);

function normalizeModule(source, extraModules) {
  if (!source) return null;
  if (extraModules?.has(source) || BUILTIN_EXTERNAL.has(source)) return source;
  const bare = source.replace(/^node:/, "");
  return NODE_MODULES.has(bare) ? bare : null;
}

function memberBinding(object, member) {
  return object && member && !member.includes(".") ? `${object}.${member}` : null;
}

function importedBinding(definition, extraModules) {
  const { node, parent } = definition;
  if (node.importKind === "type" || parent.importKind === "type") return null;
  const source = normalizeModule(parent.source.value, extraModules);
  if (!source || node.type !== "ImportSpecifier") return source;
  const name = importSpecifierName(node);
  return name === "default" ? source : memberBinding(source, name);
}

function destructuredMember(pattern, name) {
  const prop = pattern.properties.find(
    (item) =>
      item.type === "Property" && item.value.type === "Identifier" && item.value.name === name,
  );
  return prop ? (prop.computed ? literalString(prop.key) : prop.key.name) : null;
}

function createBindingResolver(context, extraModules) {
  const bindings = new WeakMap();
  function resolve(input) {
    const node = unwrapExpression(input);
    if (node?.type === "CallExpression") {
      return normalizeModule(requireSource(node, context), extraModules);
    }
    if (node?.type === "MemberExpression") {
      return memberBinding(resolve(node.object), memberPropertyName(node));
    }
    if (node?.type !== "Identifier") return null;
    const variable = resolveVariable(node, context);
    if (!variable?.defs.length) {
      return FRAMEWORK_GLOBALS.has(node.name) && !isReassigned(node, context)
        ? FRAMEWORK_GLOBALS.get(node.name)
        : null;
    }
    if (bindings.has(variable)) return bindings.get(variable);
    bindings.set(variable, null);
    const def = variable.defs[0];
    let binding = null;
    if (def.type === "ImportBinding") {
      binding = importedBinding(def, extraModules);
    } else if (def.type === "Variable" && def.parent.kind === "const") {
      const { id, init } = def.node;
      const value = resolve(init);
      if (id.type === "Identifier") binding = value;
      else if (value && id.type === "ObjectPattern") {
        binding = memberBinding(value, destructuredMember(id, node.name));
      }
    }
    bindings.set(variable, binding);
    return binding;
  }
  return resolve;
}

const MOCKED_FS_MODULES = new Set(["fs", "fs/promises", "fs-extra"]);

function isFilesystemMock(node, binding) {
  if (!FRAMEWORK_MOCKS.has(binding)) return false;
  return MOCKED_FS_MODULES.has(normalizeModule(literalString(unwrapExpression(node.arguments[0]))));
}

function isUnshadowedGlobal(node, name, context) {
  return (
    node?.type === "Identifier" &&
    node.name === name &&
    !resolveVariable(node, context)?.defs.length
  );
}

module.exports = {
  createBindingResolver,
  isFilesystemMock,
  isUnshadowedGlobal,
};
