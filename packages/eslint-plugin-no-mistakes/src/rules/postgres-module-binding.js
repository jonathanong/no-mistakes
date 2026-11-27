"use strict";

const { unwrapExpression } = require("./async-ast");
const {
  isReassigned,
  stableConstInitializer,
  staticImportSource,
} = require("./async-target-bindings");
const { requireBinding } = require("./async-target-sources");
const { resolveVariable } = require("./postgres-query-text");
const { staticKeyName } = require("./postgres-no-async-sql-builder-types");

function resolved(kind, source, name) {
  return name === undefined
    ? { status: "resolved", kind, source }
    : { status: "resolved", kind, source, name };
}

function unresolved(kind, name, source) {
  const binding = { status: "unresolved", kind };
  if (name !== undefined) binding.name = name;
  if (source !== undefined) binding.source = source;
  return binding;
}

function classifyRequire(node, context) {
  const expression = unwrapExpression(node);
  const binding = requireBinding(expression, context);
  if (!binding) return null;
  // requireBinding keeps the first literal. A second argument is not Node's require.
  if (expression.arguments.length !== 1) return unresolved("namespace");
  return binding.status === "resolved"
    ? resolved("namespace", binding.source)
    : unresolved("namespace");
}

function importEqualsBinding(definition) {
  const specifier = definition.node;
  if (specifier?.type !== "TSImportEqualsDeclaration" || specifier.importKind === "type") {
    return null;
  }
  if (specifier.moduleReference?.type !== "TSExternalModuleReference") return null;
  const source = staticImportSource(definition);
  return source ? resolved("namespace", source) : unresolved("namespace");
}

function specifierBinding(definition) {
  if (definition?.type !== "ImportBinding") return null;
  const specifier = definition.node;
  if (specifier?.type === "TSImportEqualsDeclaration") return importEqualsBinding(definition);
  const source = staticImportSource(definition);
  if (!source) return null;
  if (specifier.type === "ImportNamespaceSpecifier") return resolved("namespace", source);
  if (specifier.type === "ImportDefaultSpecifier") return resolved("default", source);
  if (specifier.type !== "ImportSpecifier") return null;
  const imported = specifier.imported;
  const name = imported?.type === "Identifier" ? imported.name : String(imported?.value);
  if (!name) return null;
  return resolved(name === "default" ? "default" : "named", source, name);
}

function moduleObject(binding) {
  return binding?.kind === "namespace" || binding?.kind === "default";
}

function followedBinding(binding) {
  if (!binding || (binding.kind !== "named" && !moduleObject(binding))) return null;
  return binding.status === "unresolved"
    ? unresolved(binding.kind, binding.name, binding.source)
    : binding;
}

function destructuredBinding(definition, declarator, context, seen) {
  const property = declarator.id.properties.find(
    (item) =>
      item.type === "Property" &&
      (item.value === definition.name || item.value?.left === definition.name),
  );
  if (!property || property.value?.type === "AssignmentPattern") return null;
  const key = staticKeyName(property);
  if (!key) return null;
  if (definition.parent?.kind !== "const" || isReassigned(definition.name, context)) return null;
  const object = resolveModuleBinding(declarator.init, context, seen);
  if (!moduleObject(object)) return null;
  if (object.status === "unresolved") return unresolved("named", key, object.source);
  return {
    status: "resolved",
    kind: "named",
    source: object.source,
    name: key,
    via: "destructure",
  };
}

function resolveModuleBinding(node, context, seen = new Set()) {
  const expression = unwrapExpression(node);
  if (!expression) return null;
  const required = classifyRequire(expression, context);
  if (required) return required;
  if (expression.type !== "Identifier") return null;
  const variable = resolveVariable(expression, context);
  const definition = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (!definition || seen.has(definition)) return null;
  seen.add(definition);
  const direct = specifierBinding(definition);
  if (direct) {
    return direct.status === "resolved" && isReassigned(expression, context)
      ? unresolved(direct.kind, direct.name, direct.source)
      : direct;
  }
  if (definition.type !== "Variable" || !definition.node) return null;
  const declarator = definition.node;
  if (declarator.id?.type === "ObjectPattern") {
    return destructuredBinding(definition, declarator, context, seen);
  }
  if (declarator.id?.type !== "Identifier") return null;
  const init = stableConstInitializer(expression, context, seen);
  if (!init) return null;
  return followedBinding(resolveModuleBinding(init, context, seen));
}

function moduleNamespace(node, context, options, seen = new Set()) {
  const binding = resolveModuleBinding(node, context, seen);
  if (!moduleObject(binding) || binding.kind !== "namespace") return false;
  if (binding.status === "unresolved") {
    if (options?.importSpecifier && binding.source && binding.source !== options.importSpecifier) {
      return false;
    }
    return "unresolved";
  }
  if (options?.importSpecifier && binding.source !== options.importSpecifier) return false;
  return true;
}

module.exports = { moduleNamespace, resolveModuleBinding };
