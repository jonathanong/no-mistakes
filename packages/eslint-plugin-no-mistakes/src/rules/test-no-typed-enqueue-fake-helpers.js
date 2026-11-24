"use strict";

const { memberPropertyName } = require("./logical-call");
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
  return def?.type === "ImportBinding" && def.parent?.type === "ImportDeclaration" ? def : null;
}

function runtimeImport(def) {
  return def && def.node.importKind !== "type" && def.parent.importKind !== "type";
}

function typeArgParams(node) {
  return (node?.typeArguments ?? node?.typeParameters)?.params;
}

function namespaceImport(identifier, sourceCode) {
  const def = importDefinition(referenceBinding(identifier, sourceCode));
  return def?.node.type === "ImportNamespaceSpecifier" ? def : null;
}

function isFrameworkReceiver(receiver, sourceCode) {
  if (receiver.type === "Identifier") {
    const binding = referenceBinding(receiver, sourceCode);
    if (!binding?.defs.length) return frameworkModules.has(receiver.name);
    const def = importDefinition(binding);
    return (
      runtimeImport(def) &&
      frameworkModules.get(importSpecifierName(def.node)) === def.parent.source.value
    );
  }
  if (receiver.type !== "MemberExpression" || receiver.object.type !== "Identifier") return false;
  const def = namespaceImport(receiver.object, sourceCode);
  return (
    runtimeImport(def) &&
    frameworkModules.get(memberPropertyName(receiver)) === def.parent.source.value
  );
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
  const def = namespaceImport(typeName.left, sourceCode);
  return Boolean(def) && mockTypeModules.has(def.parent.source.value);
}

function resolveImportTarget(node, sourceCode) {
  if (node?.type === "Identifier") {
    const target = importDefinition(referenceBinding(node, sourceCode));
    if (!target || target.node.type === "ImportNamespaceSpecifier") return null;
    return {
      name: importSpecifierName(target.node) ?? "default",
      module: target.parent.source.value,
    };
  }
  if (
    node?.type === "TSQualifiedName" &&
    node.left.type === "Identifier" &&
    node.right.type === "Identifier"
  ) {
    const target = namespaceImport(node.left, sourceCode);
    return target ? { name: node.right.name, module: target.parent.source.value } : null;
  }
  if (node?.type === "MemberExpression" && node.object.type === "Identifier") {
    const name = memberPropertyName(node);
    const target = name ? namespaceImport(node.object, sourceCode) : null;
    return target ? { name, module: target.parent.source.value } : null;
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
