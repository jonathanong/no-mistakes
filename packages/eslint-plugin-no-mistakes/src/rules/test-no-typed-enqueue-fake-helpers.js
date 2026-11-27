"use strict";

const { unwrapExpression } = require("./async-ast");
const { isReassigned, staticImportSource } = require("./async-target-bindings");
const { requireSource } = require("./async-target-sources");
const { literalString, memberPropertyName } = require("./logical-call");
const { importSpecifierName } = require("./module-mock-helpers");

const frameworkModules = new Map([
  ["vi", "vitest"],
  ["jest", "@jest/globals"],
]);
const mockTypeNames = new Set(["Mock", "MockedFunction"]);
const mockTypeModules = new Set(["vitest", "@jest/globals", "jest-mock"]);

function referenceBinding(identifier, sourceCode) {
  const scope = sourceCode.getScope(identifier);
  return scope.references.find((reference) => reference.identifier === identifier)?.resolved;
}

function importDefinition(binding) {
  const def = binding?.defs.length === 1 ? binding.defs[0] : null;
  return def?.type === "ImportBinding" &&
    (def.parent?.type === "ImportDeclaration" || def.parent?.type === "TSImportEqualsDeclaration")
    ? def
    : null;
}

function runtimeImport(def) {
  return def && def.node.importKind !== "type" && def.parent.importKind !== "type";
}

function typeArgParams(node) {
  return (node?.typeArguments ?? node?.typeParameters)?.params;
}

function scopeContext(sourceCode) {
  return { sourceCode };
}

function namespaceSource(def, allowTypeOnly) {
  if (!allowTypeOnly) return staticImportSource(def);
  if (def.node.type === "TSImportEqualsDeclaration") {
    const reference = def.node.moduleReference;
    return reference?.type === "TSExternalModuleReference"
      ? literalString(reference.expression)
      : null;
  }
  const source = def.parent?.source?.value;
  return typeof source === "string" ? source : null;
}

function namespaceModule(identifier, sourceCode, allowTypeOnly = false) {
  const binding = referenceBinding(identifier, sourceCode);
  const def = importDefinition(binding);
  if (
    def &&
    (def.node.type === "ImportNamespaceSpecifier" ||
      def.node.type === "TSImportEqualsDeclaration" ||
      def.node.type === "ImportDefaultSpecifier") &&
    (allowTypeOnly || runtimeImport(def))
  ) {
    return namespaceSource(def, allowTypeOnly);
  }
  const variable = binding?.defs.length === 1 ? binding.defs[0] : null;
  if (
    variable?.type === "Variable" &&
    variable.parent?.kind === "const" &&
    variable.node?.id?.type === "Identifier"
  ) {
    return requireSource(variable.node.init, scopeContext(sourceCode));
  }
  return null;
}

function patternExportName(pattern, name) {
  for (const property of pattern?.properties || []) {
    if (property.type !== "Property") continue;
    let value = property.value;
    while (value?.type === "AssignmentPattern") value = value.left;
    if (value?.type !== "Identifier" || value.name !== name) continue;
    if (property.computed) continue;
    if (property.key?.type === "Identifier") return property.key.name;
    return literalString(property.key);
  }
  return null;
}

function isFrameworkReceiver(receiver, sourceCode) {
  receiver = unwrapExpression(receiver);
  if (receiver?.type === "Identifier") {
    const binding = referenceBinding(receiver, sourceCode);
    if (!binding?.defs.length) return frameworkModules.has(receiver.name);
    const def = importDefinition(binding);
    if (
      runtimeImport(def) &&
      frameworkModules.get(importSpecifierName(def.node)) === staticImportSource(def)
    ) {
      return true;
    }
    const variable = binding?.defs.length === 1 ? binding.defs[0] : null;
    if (variable?.type !== "Variable" || variable.node?.id?.type !== "ObjectPattern") return false;
    if (isReassigned(receiver, scopeContext(sourceCode))) return false;
    const init = unwrapExpression(variable.node.init);
    const source =
      requireSource(variable.node.init, scopeContext(sourceCode)) ||
      (init?.type === "Identifier" ? namespaceModule(init, sourceCode) : null);
    return frameworkModules.get(patternExportName(variable.node.id, receiver.name)) === source;
  }
  if (receiver?.type !== "MemberExpression") return false;
  const object = unwrapExpression(receiver.object);
  if (object?.type !== "Identifier") return false;
  return frameworkModules.get(memberPropertyName(receiver)) === namespaceModule(object, sourceCode);
}

function isFrameworkMockType(typeName, sourceCode) {
  if (typeName?.type === "Identifier") {
    const binding = referenceBinding(typeName, sourceCode);
    if (!binding?.defs.length) return mockTypeNames.has(typeName.name);
    const def = importDefinition(binding);
    return (
      Boolean(def) &&
      def.node.type !== "ImportNamespaceSpecifier" &&
      mockTypeNames.has(importSpecifierName(def.node)) &&
      mockTypeModules.has(def.parent.source.value)
    );
  }
  if (
    typeName?.type !== "TSQualifiedName" ||
    typeName.left.type !== "Identifier" ||
    typeName.right.type !== "Identifier" ||
    !mockTypeNames.has(typeName.right.name)
  ) {
    return false;
  }
  const binding = referenceBinding(typeName.left, sourceCode);
  if (!binding?.defs.length) return frameworkModules.has(typeName.left.name);
  return mockTypeModules.has(namespaceModule(typeName.left, sourceCode, true));
}

function resolveImportTarget(node, sourceCode) {
  node = unwrapExpression(node);
  if (node?.type === "Identifier") {
    const binding = referenceBinding(node, sourceCode);
    const target = importDefinition(binding);
    if (
      target &&
      target.node.type !== "ImportNamespaceSpecifier" &&
      target.node.type !== "TSImportEqualsDeclaration"
    ) {
      const typedSource = target.parent?.source?.value;
      const module =
        staticImportSource(target) || (typeof typedSource === "string" ? typedSource : null);
      return module ? { name: importSpecifierName(target.node) ?? "default", module } : null;
    }
    const variable = binding?.defs.length === 1 ? binding.defs[0] : null;
    if (variable?.type === "Variable" && variable.node?.id?.type === "ObjectPattern") {
      if (isReassigned(node, scopeContext(sourceCode))) return null;
      const name = patternExportName(variable.node.id, node.name);
      const init = unwrapExpression(variable.node.init);
      const module =
        requireSource(variable.node.init, scopeContext(sourceCode)) ||
        (init?.type === "Identifier" ? namespaceModule(init, sourceCode) : null);
      return name && module ? { name, module } : null;
    }
    return null;
  }
  if (
    node?.type === "TSQualifiedName" &&
    node.left.type === "Identifier" &&
    node.right.type === "Identifier"
  ) {
    const module = namespaceModule(node.left, sourceCode, true);
    return module ? { name: node.right.name, module } : null;
  }
  if (node?.type === "MemberExpression") {
    const object = unwrapExpression(node.object);
    const name = memberPropertyName(node);
    const module =
      object?.type === "Identifier" && name ? namespaceModule(object, sourceCode) : null;
    return module ? { name, module } : null;
  }
  return null;
}

module.exports = {
  isFrameworkMockType,
  isFrameworkReceiver,
  referenceBinding,
  resolveImportTarget,
  typeArgParams,
};
