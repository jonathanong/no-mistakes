"use strict";

const { unwrapExpression, unwrapSequence } = require("./async-ast");
const { requireResolution, requireSource } = require("./async-target-sources");
const { literalString, logicalCall } = require("./logical-call");
const { isUnresolved, moduleLoad } = require("./module-load");
const { resolveVariable } = require("./test-no-shared-state-aliases");

const TEST_NAMES = new Set(["test", "it", "describe"]);

function staticName(node, computed) {
  node = unwrapExpression(node);
  if (!node) return null;
  if (computed && node.type === "Identifier") return null;
  if (!computed && typeof node.name === "string" && node.type !== "Literal") return node.name;
  return literalString(node);
}

function hasSequentialOption(node) {
  // Opaque forwarding must not erase options already visible to the legacy check.
  const args =
    node.type === "CallExpression"
      ? (logicalCall(node)?.arguments ?? node.arguments)
      : node.arguments;
  return (args || []).slice(1).some((argument) => {
    const value = unwrapSequence(argument);
    return (
      value?.type === "ObjectExpression" &&
      value.properties.some((property) => {
        if (property.type !== "Property") return false;
        const key = staticName(property.key, property.computed);
        const option = unwrapExpression(property.value);
        return key === "sequential" && option?.type === "Literal" && option.value === true;
      })
    );
  });
}

function importedName(specifier) {
  const imported = specifier.imported;
  return imported.type === "Literal" ? String(imported.value) : imported.name;
}

function patternTests(pattern) {
  const found = [];
  for (const property of pattern?.properties || []) {
    if (property.type !== "Property") continue;
    let value = property.value;
    while (value?.type === "AssignmentPattern") value = value.left;
    if (value?.type !== "Identifier") continue;
    const key = property.computed
      ? literalString(property.key)
      : property.key?.type === "Identifier"
        ? property.key.name
        : literalString(property.key);
    if (key === null) found.push({ dynamic: true, local: value });
    else if (TEST_NAMES.has(key)) found.push({ key, local: value });
  }
  return found;
}

function recordVitestImport(node, state) {
  if (node.source?.value !== "vitest" || node.importKind === "type") return;
  state.usesVitest = true;
  for (const specifier of node.specifiers) {
    if (specifier.importKind === "type") continue;
    if (
      specifier.type === "ImportNamespaceSpecifier" ||
      specifier.type === "ImportDefaultSpecifier"
    ) {
      state.namespaces.add(specifier.local.name);
    }
    if (specifier.type === "ImportSpecifier" && TEST_NAMES.has(importedName(specifier))) {
      state.roots.add(specifier.local.name);
    }
  }
}

function recordImportEquals(node, state) {
  if (node?.importKind === "type" || node.moduleReference?.type !== "TSExternalModuleReference") {
    return;
  }
  const source = literalString(node.moduleReference.expression);
  if (source !== "vitest") return;
  state.usesVitest = true;
  state.namespaces.add(node.id.name);
}

function moduleScope(identifier, context) {
  const scope = resolveVariable(identifier, context)?.scope;
  const kind = scope?.type;
  return (
    kind === "module" ||
    kind === "global" ||
    kind === "script" ||
    (kind === "function" && scope.block?.type === "Program")
  );
}

function recordRequire(declaration, state, context) {
  const resolution = requireResolution(declaration?.init, context);
  if (!resolution) return;
  if (resolution.source !== null && resolution.source !== "vitest") return;
  state.usesVitest = true;
  // A non-static require is judged later as unanalyzable, not as a runner.
  if (resolution.source === null) return;
  if (declaration.id?.type === "Identifier") {
    state.namespaces.add(declaration.id.name);
    return;
  }
  for (const entry of patternTests(declaration.id)) {
    if (!entry.dynamic && moduleScope(entry.local, context)) state.roots.add(entry.local.name);
  }
}

function hasLaterWrite(variable) {
  return variable.references.some((reference) => reference.isWrite() && !reference.init);
}

function bindingRoot(node) {
  let current = unwrapExpression(node);
  while (current?.type === "MemberExpression" || current?.type === "CallExpression") {
    current = unwrapExpression(current.type === "CallExpression" ? current.callee : current.object);
  }
  return current?.type === "Identifier" ? current : null;
}

function reassignedVitestRequire(callee, context) {
  const root = bindingRoot(callee);
  if (!root) return false;
  const variable = resolveVariable(root, context);
  if (!variable || variable.defs.length !== 1) return false;
  const def = variable.defs[0];
  return (
    def.type === "Variable" &&
    def.node?.id?.type === "Identifier" &&
    requireSource(def.node.init, context) === "vitest" &&
    hasLaterWrite(variable)
  );
}

function dynamicLetRequire(callee, context) {
  const root = bindingRoot(callee);
  if (!root) return false;
  const variable = resolveVariable(root, context);
  if (!variable || variable.defs.length !== 1) return false;
  const def = variable.defs[0];
  return (
    def.type === "Variable" &&
    def.parent?.kind !== "const" &&
    def.node?.id?.type === "Identifier" &&
    requireResolution(def.node.init, context)?.source === null
  );
}

function unresolvedSequential(callee, context) {
  const target = unwrapSequence(callee);
  return (
    isUnresolved(moduleLoad(target, context)) ||
    reassignedVitestRequire(target, context) ||
    dynamicLetRequire(target, context)
  );
}

function isUnshadowedNamespace(identifier, context, namespaces) {
  if (!namespaces.has(identifier?.name)) return false;
  const variable = resolveVariable(identifier, context);
  if (!variable || variable.defs.length === 0) return true;
  if (variable.defs.length !== 1) return false;
  const def = variable.defs[0];
  if (def.type === "ImportBinding") {
    return [
      "ImportNamespaceSpecifier",
      "ImportDefaultSpecifier",
      "TSImportEqualsDeclaration",
    ].includes(def.node?.type);
  }
  return (
    def.type === "Variable" &&
    def.node?.id?.type === "Identifier" &&
    requireSource(def.node.init, context) === "vitest" &&
    !hasLaterWrite(variable)
  );
}

function destructuredFromNamespace(init, context, namespaces) {
  const value = unwrapExpression(init);
  return value?.type === "Identifier" && isUnshadowedNamespace(value, context, namespaces);
}

module.exports = {
  destructuredFromNamespace,
  hasSequentialOption,
  isUnshadowedNamespace,
  recordImportEquals,
  recordRequire,
  recordVitestImport,
  staticName,
  unresolvedSequential,
};
