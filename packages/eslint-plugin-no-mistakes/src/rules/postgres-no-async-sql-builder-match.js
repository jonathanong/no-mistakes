"use strict";

const { traverse, unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { isInside } = require("./postgres-no-async-sql-builder-bindings");
const { staticKeyName } = require("./postgres-no-async-sql-builder-types");

function stringList(value) {
  return value || [];
}

function configuredBuilders(option) {
  const entries = option?.builders;
  if (!entries || entries.length === 0) return [];
  return entries
    .map((entry) => ({
      names: stringList(entry.names),
      callees: stringList(entry.callees),
      source: entry.source || null,
    }))
    .filter((entry) => entry.names.length > 0 || entry.callees.length > 0);
}

function functionName(func) {
  const parent = func.parent;
  if (
    parent?.type === "VariableDeclarator" &&
    parent.init === func &&
    parent.id.type === "Identifier"
  ) {
    return parent.id.name;
  }
  if (func.id?.type === "Identifier") return func.id.name;
  if (
    (parent?.type === "MethodDefinition" ||
      parent?.type === "PropertyDefinition" ||
      parent?.type === "Property") &&
    parent.value === func
  ) {
    return staticKeyName(parent);
  }
  if (
    parent?.type === "AssignmentExpression" &&
    parent.right === func &&
    parent.left.type === "Identifier"
  ) {
    return parent.left.name;
  }
  return null;
}

function importedBinding(identifier, context) {
  if (identifier?.type !== "Identifier") return null;
  const variable = resolveVariable(identifier, context);
  const definition = variable?.defs.find(
    (item) => item.type === "ImportBinding" && item.node.importKind !== "type",
  );
  const specifier = definition?.node;
  const declaration = specifier?.parent;
  if (
    !specifier ||
    declaration?.type !== "ImportDeclaration" ||
    declaration.importKind === "type" ||
    typeof declaration.source?.value !== "string"
  ) {
    return null;
  }
  if (specifier.type === "ImportNamespaceSpecifier") {
    return { kind: "namespace", source: declaration.source.value };
  }
  if (specifier.type !== "ImportSpecifier") return null;
  const imported = specifier.imported;
  const name = imported.type === "Identifier" ? imported.name : String(imported.value);
  return { kind: "named", source: declaration.source.value, name };
}

function calleeMatches(callee, builder, context) {
  const node = unwrapExpression(callee);
  if (node.type === "Identifier") {
    if (!builder.source) return builder.callees.includes(node.name);
    const imported = importedBinding(node, context);
    return Boolean(
      imported &&
      imported.kind === "named" &&
      imported.source === builder.source &&
      builder.callees.includes(imported.name),
    );
  }
  if (node.type !== "MemberExpression") return false;
  const name = staticKeyName(node);
  if (!name || !builder.callees.includes(name)) return false;
  if (!builder.source) return true;
  const imported = importedBinding(unwrapExpression(node.object), context);
  return Boolean(imported && imported.kind === "namespace" && imported.source === builder.source);
}

function localInitializer(identifier, func, context) {
  const variable = resolveVariable(identifier, context);
  if (!variable || variable.defs.length !== 1) return null;
  const definition = variable.defs[0];
  if (definition.type !== "Variable" || definition.parent?.kind === "var") return null;
  if (definition.node.id?.type !== "Identifier" || !definition.node.init) return null;
  if (!isInside(definition.node, func)) return null;
  const writes = variable.references.some((reference) => reference.isWrite() && !reference.init);
  return writes ? null : definition.node.init;
}

function expressionUsesCallee(node, builder, context, func, seen) {
  const current = unwrapExpression(node);
  if (current.type === "ConditionalExpression") {
    return (
      expressionUsesCallee(current.consequent, builder, context, func, seen) ||
      expressionUsesCallee(current.alternate, builder, context, func, seen)
    );
  }
  if (current.type === "LogicalExpression") {
    return (
      expressionUsesCallee(current.left, builder, context, func, seen) ||
      expressionUsesCallee(current.right, builder, context, func, seen)
    );
  }
  if (current.type === "SequenceExpression") {
    return expressionUsesCallee(
      current.expressions[current.expressions.length - 1],
      builder,
      context,
      func,
      seen,
    );
  }
  if (current.type === "CallExpression" || current.type === "TaggedTemplateExpression") {
    const callee = current.type === "CallExpression" ? current.callee : current.tag;
    if (calleeMatches(callee, builder, context)) return true;
    const unwrapped = unwrapExpression(callee);
    return unwrapped?.type === "MemberExpression"
      ? expressionUsesCallee(unwrapped.object, builder, context, func, seen)
      : false;
  }
  if (current.type === "MemberExpression") {
    return expressionUsesCallee(current.object, builder, context, func, seen);
  }
  if (current.type !== "Identifier" || seen.has(current)) return false;
  const initializer = localInitializer(current, func, context);
  if (!initializer) return false;
  seen.add(current);
  return expressionUsesCallee(initializer, builder, context, func, seen);
}

function returnExpressions(context, func) {
  if (func.body.type !== "BlockStatement") return [func.body];
  const expressions = [];
  traverse(context, func.body, (node) => {
    if (node.type === "ReturnStatement" && node.argument) expressions.push(node.argument);
  });
  return expressions;
}

function returnsConfiguredCallee(func, builder, context) {
  return returnExpressions(context, func).some((expression) =>
    expressionUsesCallee(expression, builder, context, func, new Set()),
  );
}

function isConfiguredBuilder(func, builders, context) {
  const name = functionName(func);
  return builders.some((builder) => {
    if (name && builder.names.includes(name)) return true;
    return builder.callees.length > 0 && returnsConfiguredCallee(func, builder, context);
  });
}

module.exports = {
  configuredBuilders,
  isConfiguredBuilder,
};
