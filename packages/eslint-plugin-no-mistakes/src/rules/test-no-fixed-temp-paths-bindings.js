"use strict";

const { unwrapExpression } = require("./async-ast");
const { filesystemPathCount } = require("./test-no-fixed-temp-paths-sinks");
const { literalString } = require("./logical-call");
const { isReassigned, memberPropertyName, resolveVariable } = require("./async-target-bindings");
const { importSpecifierName, requireResolution, requireSource } = require("./async-target-sources");

const UNRESOLVED = Symbol("unresolved");
const NODE_MODULES = new Set(["path", "os", "fs", "fs/promises", "child_process"]);
const PATH_METHODS = new Set([
  "appendFile",
  "copyFile",
  "cp",
  "exists",
  "existsSync",
  "lstat",
  "mkdir",
  "open",
  "readFile",
  "realpath",
  "rename",
  "rm",
  "rmdir",
  "stat",
  "unlink",
  "writeFile",
]);
const PROCESS_METHODS = new Set(["exec", "execFile", "spawn"]);
const FRAMEWORK_GLOBALS = new Map([
  ["vi", "vitest.vi"],
  ["jest", "@jest/globals.jest"],
]);
const FRAMEWORK_MODULES = new Set(["vitest", "@jest/globals"]);
const FRAMEWORK_MOCKS = new Set(["vitest.vi.mock", "@jest/globals.jest.mock"]);

function supportedModule(source, extraModules) {
  if (!source) return null;
  if (FRAMEWORK_MODULES.has(source) || extraModules?.has(source)) return source;
  const name = source.replace(/^node:/, "");
  return NODE_MODULES.has(name) || extraModules?.has(name) ? name : null;
}

function memberBinding(object, member) {
  return object && member && !member.includes(".") ? `${object}.${member}` : null;
}

function unresolvedMember(member) {
  if (member === null || member === "promises") return UNRESOLVED;
  const bare = member.replace(/Sync$/, "");
  const candidate =
    PATH_METHODS.has(bare) || PROCESS_METHODS.has(bare) || filesystemPathCount("fs", member) > 0;
  return candidate ? UNRESOLVED : null;
}

function importedBinding(definition, extraModules) {
  const { node, parent } = definition;
  if (node.importKind === "type" || parent.importKind === "type") return null;
  if (parent.type === "TSImportEqualsDeclaration") {
    const reference = parent.moduleReference;
    if (reference.type !== "TSExternalModuleReference") return null;
    const source = literalString(reference.expression);
    return source === null ? UNRESOLVED : supportedModule(source, extraModules);
  }
  const source = supportedModule(parent.source.value, extraModules);
  if (!source || node.type !== "ImportSpecifier") return source;
  const name = importSpecifierName(node);
  return name === "default" ? source : memberBinding(source, name);
}

function destructuredMember(pattern, name) {
  const property = pattern.properties.find(
    (item) =>
      item.type === "Property" && item.value.type === "Identifier" && item.value.name === name,
  );
  if (!property) return null;
  return !property.computed && property.key.type === "Identifier"
    ? property.key.name
    : literalString(property.key);
}

function isReassignedGlobal(node, context) {
  return (
    isReassigned(node, context) ||
    context.sourceCode.scopeManager.globalScope.through.some(
      (reference) =>
        reference.identifier.name === node.name && !reference.resolved && reference.isWrite(),
    )
  );
}

function bindingResolver(context, extraModules) {
  const bindings = new WeakMap();
  const globals = new Map();
  function resolve(input) {
    const node = unwrapExpression(input);
    if (node?.type === "CallExpression") {
      const required = requireResolution(node, context);
      if (required?.source === null) return UNRESOLVED;
      return supportedModule(requireSource(node, context), extraModules);
    }
    if (node?.type === "MemberExpression") {
      const object = resolve(node.object);
      const member = memberPropertyName(node);
      if (object === UNRESOLVED && node.computed && member == null) return null;
      if (object === UNRESOLVED) return unresolvedMember(member);
      return memberBinding(object, member);
    }
    if (node?.type !== "Identifier") return null;
    const variable = resolveVariable(node, context);
    if (!variable?.defs.length && FRAMEWORK_GLOBALS.has(node.name)) {
      if (!globals.has(node.name)) {
        globals.set(
          node.name,
          isReassignedGlobal(node, context) ? null : FRAMEWORK_GLOBALS.get(node.name),
        );
      }
      return globals.get(node.name);
    }
    if (variable?.defs.length !== 1) return null;
    if (bindings.has(variable)) return bindings.get(variable);
    bindings.set(variable, null);
    const definition = variable.defs[0];
    let binding = null;
    if (definition.type === "ImportBinding") {
      binding = importedBinding(definition, extraModules);
    } else if (definition.type === "Variable" && definition.parent.kind === "const") {
      const { id, init } = definition.node;
      const value = resolve(init);
      if (id.type === "Identifier") binding = value;
      else if (id.type === "ObjectPattern") {
        const member = destructuredMember(id, node.name);
        const unknownKey = id.properties.some(
          (item) =>
            item.type === "Property" &&
            item.computed &&
            item.value?.name === node.name &&
            literalString(item.key) == null,
        );
        if (value === UNRESOLVED) binding = unknownKey ? null : unresolvedMember(member);
        else if (value) binding = member ? memberBinding(value, member) : null;
      } else if (value === UNRESOLVED) binding = UNRESOLVED;
    }
    bindings.set(variable, binding);
    return binding;
  }
  return resolve;
}

function isFilesystemMock(node, binding, extraMockModules) {
  if (!FRAMEWORK_MOCKS.has(binding)) return false;
  const source = supportedModule(
    literalString(unwrapExpression(node.arguments[0])),
    extraMockModules,
  );
  return source === "fs" || source === "fs/promises" || Boolean(extraMockModules?.has(source));
}

function isUnshadowedGlobal(node, name, context) {
  return (
    node?.type === "Identifier" &&
    node.name === name &&
    !resolveVariable(node, context)?.defs.length
  );
}

module.exports = { UNRESOLVED, bindingResolver, isFilesystemMock, isUnshadowedGlobal };
