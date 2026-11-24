"use strict";

const { unwrapExpression } = require("./async-ast");
const { resolveVariable, stableConstInitializer } = require("./async-target-bindings");
const { requireSource } = require("./async-target-sources");
const { memberPropertyName } = require("./logical-call");
const { importedBinding } = require("./postgres-no-async-sql-builder-import");
const { staticKeyName } = require("./postgres-no-async-sql-builder-types");

function namespaceSource(node, context) {
  const expression = unwrapExpression(node);
  const imported = importedBinding(expression, context);
  if (imported?.kind === "default" || imported?.kind === "namespace") return imported.source;
  return (
    requireSource(expression, context) ??
    requireSource(stableConstInitializer(expression, context, new Set()), context)
  );
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

function isPoolConstructor(node, context, modules) {
  const callee = unwrapExpression(node.callee);
  if (callee.type === "Identifier") {
    const imported = importedBinding(callee, context);
    const source = imported?.name === "Pool" ? imported.source : localPoolSource(callee, context);
    return modules.has(source);
  }
  return modules.has(poolMemberSource(callee, context));
}

module.exports = { isPoolConstructor };
