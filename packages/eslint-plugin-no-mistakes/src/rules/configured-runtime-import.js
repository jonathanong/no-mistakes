"use strict";

const { unwrapExpression } = require("./async-ast");
const { memberPropertyName, resolveVariable } = require("./async-target-bindings");
const { requireSource } = require("./async-target-sources");

const importSelectorSchema = {
  type: "object",
  properties: {
    source: { type: "string", minLength: 1 },
    imported: { type: "string", minLength: 1 },
  },
  required: ["source", "imported"],
  additionalProperties: false,
};

function oneDef(id, context) {
  const variable = resolveVariable(id, context);
  if (!variable || variable.defs.length !== 1) return null;
  return { variable, def: variable.defs[0] };
}

function importEqualsSource(node) {
  const ref = node?.moduleReference;
  if (node?.type !== "TSImportEqualsDeclaration" || node.importKind === "type") return null;
  if (ref?.type !== "TSExternalModuleReference") return null;
  // The external-module grammar only accepts a string literal.
  return String(unwrapExpression(ref.expression).value);
}

function patternName(pattern, localName) {
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const value =
      property.value?.type === "AssignmentPattern" ? property.value.left : property.value;
    if (value?.type !== "Identifier" || value.name !== localName) continue;
    return memberPropertyName({
      type: "MemberExpression",
      computed: Boolean(property.computed),
      property: property.key,
    });
  }
  return null;
}

function resolveModuleObject(expression, context, seen) {
  const node = unwrapExpression(expression);
  if (!node) return null;
  const required = requireSource(node, context);
  if (required) return { source: required, kind: "module" };
  if (node.type !== "Identifier") return null;
  const found = oneDef(node, context);
  if (!found || seen.has(found.variable)) return null;
  const { def } = found;
  const equals = importEqualsSource(def.node);
  if (equals) return { source: equals, kind: "namespace" };
  if (
    def.type === "ImportBinding" &&
    (def.node?.type === "ImportNamespaceSpecifier" ||
      def.node?.type === "ImportDefaultSpecifier") &&
    def.parent?.importKind !== "type"
  ) {
    return {
      source: def.parent.source.value,
      kind: def.node.type === "ImportDefaultSpecifier" ? "module" : "namespace",
    };
  }
  if (
    def.type === "Variable" &&
    def.parent?.kind === "const" &&
    def.node?.id?.type === "Identifier"
  ) {
    seen.add(found.variable);
    return resolveModuleObject(def.node.init, context, seen);
  }
  return null;
}

function resolveImportedValue(expression, context, seen = new Set()) {
  const node = unwrapExpression(expression);
  if (!node) return null;
  if (node.type === "MemberExpression") {
    const base = resolveModuleObject(node.object, context, seen);
    if (!base) return null;
    const name = memberPropertyName(node);
    return name
      ? { source: base.source, imported: name }
      : { unresolved: true, source: base.source };
  }
  if (node.type !== "Identifier") return null;
  const found = oneDef(node, context);
  if (!found || seen.has(found.variable)) return null;
  const { def } = found;
  if (
    def.type === "ImportBinding" &&
    def.node?.type === "ImportSpecifier" &&
    def.node.importKind !== "type" &&
    def.parent?.importKind !== "type"
  ) {
    const importedName =
      def.node.imported.type === "Literal"
        ? String(def.node.imported.value)
        : def.node.imported.name;
    const local = def.node.local.name;
    return {
      source: def.parent.source.value,
      imported: importedName,
      names: importedName === "default" ? [local, "default"] : [importedName],
    };
  }
  if (
    def.type === "ImportBinding" &&
    def.node?.type === "ImportDefaultSpecifier" &&
    def.parent?.importKind !== "type"
  ) {
    const local = def.node.local.name;
    return {
      source: def.parent.source.value,
      imported: "default",
      names: [local, "default"],
    };
  }
  if (def.type === "Variable" && def.node?.id?.type === "ObjectPattern") {
    const name = patternName(def.node.id, node.name);
    const base = resolveModuleObject(def.node.init, context, seen);
    if (!base) return null;
    return name
      ? { source: base.source, imported: name }
      : { unresolved: true, source: base.source };
  }
  if (
    def.type === "Variable" &&
    def.parent?.kind === "const" &&
    def.node?.id?.type === "Identifier"
  ) {
    seen.add(found.variable);
    const base = resolveModuleObject(def.node.init, context, seen);
    if (base?.kind === "module") return { source: base.source, imported: "default" };
  }
  return null;
}

function matchesRuntimeImport(expression, selector, context) {
  const resolved = resolveImportedValue(expression, context);
  return Boolean(
    resolved &&
    !resolved.unresolved &&
    resolved.source === selector.source &&
    resolved.imported === selector.imported,
  );
}

module.exports = { importSelectorSchema, matchesRuntimeImport, resolveImportedValue };
