"use strict";

const { unwrapExpression } = require("./async-ast");
const {
  memberPropertyName,
  resolveVariable,
  staticImportSource,
} = require("./async-target-bindings");
const { requireResolution } = require("./async-target-sources");
const { literalString } = require("./logical-call");

function patternExport(pattern, name) {
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    let value = property.value;
    while (value?.type === "AssignmentPattern") value = value.left;
    if (value?.type === "ObjectPattern") {
      const nested = patternExport(value, name);
      if (nested !== undefined) return nested;
    }
    if (value?.type !== "Identifier" || value.name !== name) continue;
    if (!property.computed && property.key?.type === "Identifier") return property.key.name;
    return literalString(property.key);
  }
  return undefined;
}

function importLoad(definition) {
  if (definition?.type !== "ImportBinding") return null;
  const source = staticImportSource(definition);
  if (definition.node?.type === "TSImportEqualsDeclaration") {
    if (
      definition.node.importKind === "type" ||
      definition.node.moduleReference?.type !== "TSExternalModuleReference"
    ) {
      return null;
    }
    return { source, exportName: "*" };
  }
  if (!source) return null;
  if (definition.node?.type === "ImportNamespaceSpecifier") return { source, exportName: "*" };
  if (definition.node?.type === "ImportDefaultSpecifier") return { source, exportName: "default" };
  const imported = definition.node?.imported;
  const exportName = imported?.type === "Literal" ? String(imported.value) : imported?.name;
  return exportName ? { source, exportName } : { source: null };
}

// A static module, a named export of one, or `{ source: null }` when the load is real but not static.
function moduleLoad(node, context, seen = new Set()) {
  node = unwrapExpression(node);
  if (!node) return null;
  const required = requireResolution(node, context);
  if (required)
    return required.source === null
      ? { source: null }
      : { source: required.source, exportName: "*" };
  if (node.type === "MemberExpression") {
    const object = moduleLoad(node.object, context, seen);
    if (!object) return null;
    if (object.source === null) return object;
    const exportName = memberPropertyName(node);
    return exportName ? { source: object.source, exportName } : { source: null };
  }
  if (node.type !== "Identifier" || seen.has(node.name)) return null;
  const variable = resolveVariable(node, context);
  const definition = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (!definition) return null;
  seen.add(node.name);
  const imported = importLoad(definition);
  if (imported) return imported;
  if (
    definition.type !== "Variable" ||
    definition.parent?.kind !== "const" ||
    !definition.node?.init
  ) {
    return null;
  }
  const init = moduleLoad(definition.node.init, context, seen);
  if (!init) return null;
  if (definition.node.id?.type === "Identifier") return init;
  if (definition.node.id?.type !== "ObjectPattern") return null;
  if (init.source === null) return init;
  const exportName = patternExport(definition.node.id, node.name);
  // A computed key on a resolved module is a documented silent limit.
  // A non-static specifier already returned above.
  if (!exportName) return null;
  return { source: init.source, exportName };
}

function isUnresolved(load) {
  return load?.source === null;
}

module.exports = { isUnresolved, moduleLoad };
