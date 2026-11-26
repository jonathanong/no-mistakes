"use strict";

const { unwrapExpression } = require("./async-ast");
const { isLocalRequire } = require("./async-target-bindings");
const { literalString } = require("./logical-call");
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

function written(variable) {
  return variable.references.some((reference) => reference.isWrite() && !reference.init);
}

function classifyRequire(node, context) {
  if (node?.type !== "CallExpression") return null;
  const callee = unwrapExpression(node.callee);
  if (callee?.type !== "Identifier" || callee.name !== "require") return null;
  if (isLocalRequire(callee, context)) return null;
  const argument = node.arguments.length === 1 ? node.arguments[0] : null;
  const source = argument ? literalString(unwrapExpression(argument)) : null;
  return source === null ? unresolved("namespace") : resolved("namespace", source);
}

function importEqualsBinding(node) {
  if (node?.importKind === "type") return null;
  const reference = node?.moduleReference;
  if (reference?.type !== "TSExternalModuleReference") return null;
  const source = literalString(unwrapExpression(reference.expression));
  return source === null ? unresolved("namespace") : resolved("namespace", source);
}

function specifierBinding(definition) {
  if (definition?.type !== "ImportBinding") return null;
  const specifier = definition.node;
  if (specifier?.type === "TSImportEqualsDeclaration") return importEqualsBinding(specifier);
  const declaration = specifier?.parent;
  if (
    !specifier ||
    declaration?.type !== "ImportDeclaration" ||
    declaration.importKind === "type" ||
    specifier.importKind === "type" ||
    typeof declaration.source?.value !== "string"
  ) {
    return null;
  }
  const source = declaration.source.value;
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

function destructuredBinding(definition, declarator, variable, context, seen) {
  const property = declarator.id.properties.find(
    (item) =>
      item.type === "Property" &&
      (item.value === definition.name || item.value?.left === definition.name),
  );
  if (!property) return null;
  if (property.value?.type === "AssignmentPattern") return null;
  const key = staticKeyName(property);
  if (!key) return null;
  const object = resolveModuleBinding(declarator.init, context, seen);
  if (!moduleObject(object)) return null;
  if (object.status === "unresolved") return unresolved("named", key, object.source);
  if (definition.parent?.kind !== "const" || written(variable)) return null;
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
    return direct.status === "resolved" && written(variable)
      ? unresolved(direct.kind, direct.name, direct.source)
      : direct;
  }
  if (definition.type !== "Variable" || !definition.node) return null;
  const declarator = definition.node;
  if (declarator.id?.type === "ObjectPattern") {
    return destructuredBinding(definition, declarator, variable, context, seen);
  }
  if (declarator.id?.type !== "Identifier") return null;
  const init = resolveModuleBinding(declarator.init, context, seen);
  if (!moduleObject(init)) return null;
  if (init.status === "unresolved") return unresolved(init.kind, init.name, init.source);
  if (definition.parent?.kind !== "const" || written(variable)) return null;
  return init;
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
