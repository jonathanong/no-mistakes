"use strict";

const { unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { memberPropertyName } = require("./logical-call");
const { resolveModuleBinding } = require("./postgres-module-binding");
const { staticKeyName } = require("./postgres-no-async-sql-builder-types");

function namespaceSource(node, context) {
  const binding = resolveModuleBinding(node, context);
  if (!binding || (binding.kind !== "default" && binding.kind !== "namespace")) return null;
  if (binding.status === "resolved") return binding.source;
  return { unresolved: true, source: binding.source };
}

function poolMemberSource(node, context) {
  const member = unwrapExpression(node);
  return memberPropertyName(member) === "Pool" ? namespaceSource(member.object, context) : null;
}

function objectPropertyValue(node, key) {
  const object = unwrapExpression(node);
  if (object?.type !== "ObjectExpression" || key === "__proto__") return null;
  // The last property determines the value unless a later dynamic key or spread may replace it.
  for (let index = object.properties.length - 1; index >= 0; index -= 1) {
    const property = object.properties[index];
    if (property.type !== "Property") return null;
    const name = staticKeyName(property);
    if (name === null) return null;
    if (name === key) {
      return property.kind === "init" && !property.method ? property.value : null;
    }
  }
  return null;
}

function localPoolSource(identifier, context) {
  const definitions = resolveVariable(identifier, context)?.defs;
  if (definitions?.length !== 1) return null;
  const [definition] = definitions;
  if (definition.type !== "Variable" || definition.parent.kind !== "const") return null;
  const declarator = definition.node;
  if (declarator.id.type === "Identifier") return poolMemberSource(declarator.init, context);
  const property = definition.name.parent;
  if (
    property?.type !== "Property" ||
    property.value !== definition.name ||
    staticKeyName(property) !== "Pool"
  ) {
    return null;
  }
  const enclosingKeys = [];
  let pattern = property.parent;
  while (pattern?.type === "ObjectPattern" && pattern !== declarator.id) {
    const outerProperty = pattern.parent;
    if (outerProperty?.type !== "Property" || outerProperty.value !== pattern) return null;
    const key = staticKeyName(outerProperty);
    if (key === null) return null;
    enclosingKeys.push(key);
    pattern = outerProperty.parent;
  }
  if (pattern !== declarator.id) return null;
  let source = declarator.init;
  for (const key of enclosingKeys.reverse()) {
    source = objectPropertyValue(source, key);
  }
  return namespaceSource(source, context);
}

function poolSourceResult(source, modules) {
  if (source?.unresolved) {
    return source.source == null || modules.has(source.source) ? "unresolved" : false;
  }
  return modules.has(source);
}

function isPoolConstructor(node, context, modules) {
  const callee = unwrapExpression(node.callee);
  if (!callee) return false;
  if (callee.type === "Identifier") {
    const binding = resolveModuleBinding(callee, context);
    if (binding?.kind === "named" && binding.name === "Pool") {
      return poolSourceResult(
        binding.status === "unresolved"
          ? { unresolved: true, source: binding.source }
          : binding.source,
        modules,
      );
    }
    return poolSourceResult(localPoolSource(callee, context), modules);
  }
  return poolSourceResult(poolMemberSource(callee, context), modules);
}

module.exports = { isPoolConstructor };
