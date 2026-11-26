"use strict";

const { traverse, unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { isInside } = require("./postgres-no-async-sql-builder-bindings");
const { resolveModuleBinding } = require("./postgres-module-binding");
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

function outerParent(node) {
  return unwrapTransparentParent(node).parent;
}

function functionName(func) {
  const parent = outerParent(func);
  if (
    parent?.type === "VariableDeclarator" &&
    unwrapExpression(parent.init) === func &&
    parent.id.type === "Identifier"
  ) {
    return parent.id.name;
  }
  if (func.id?.type === "Identifier") return func.id.name;
  if (
    (parent?.type === "MethodDefinition" ||
      parent?.type === "PropertyDefinition" ||
      parent?.type === "Property") &&
    unwrapExpression(parent.value) === func
  ) {
    return staticKeyName(parent);
  }
  if (
    parent?.type === "AssignmentExpression" &&
    unwrapExpression(parent.right) === func &&
    parent.left.type === "Identifier"
  ) {
    return parent.left.name;
  }
  return null;
}

function bindingMatches(binding, builder, named) {
  if (!binding) return false;
  const accepted =
    binding.kind === "default" ||
    (named && (binding.kind === "named" || binding.kind === "namespace"));
  if (!accepted) return false;
  if (binding.status === "unresolved") {
    return !binding.source || binding.source === builder.source ? "unresolved" : false;
  }
  return binding.source === builder.source;
}

function calleeMatches(callee, builder, context) {
  const node = unwrapExpression(callee);
  if (node.type === "Identifier") {
    if (!builder.source) return builder.callees.includes(node.name);
    const binding = resolveModuleBinding(node, context);
    const named = binding?.kind === "named" && builder.callees.includes(binding.name);
    return bindingMatches(
      named || binding?.kind === "default" ? binding : null,
      builder,
      named || binding?.kind === "default",
    );
  }
  if (node.type !== "MemberExpression") return false;
  const name = staticKeyName(node);
  if (!name || !builder.callees.includes(name)) return false;
  if (!builder.source) return true;
  const binding = resolveModuleBinding(unwrapExpression(node.object), context);
  if (binding?.kind !== "namespace") return false;
  return bindingMatches(binding, builder, true);
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

function eitherMatch(left, right) {
  if (left === true || right === true) return true;
  if (left === "unresolved" || right === "unresolved") return "unresolved";
  return false;
}

function expressionUsesCallee(node, builder, context, func, seen) {
  const current = unwrapExpression(node);
  if (current.type === "ConditionalExpression") {
    return eitherMatch(
      expressionUsesCallee(current.consequent, builder, context, func, seen),
      expressionUsesCallee(current.alternate, builder, context, func, seen),
    );
  }
  if (current.type === "LogicalExpression") {
    return eitherMatch(
      expressionUsesCallee(current.left, builder, context, func, seen),
      expressionUsesCallee(current.right, builder, context, func, seen),
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
    const matched = calleeMatches(callee, builder, context);
    if (matched === true || matched === "unresolved") return matched;
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
  let unresolved = false;
  for (const expression of returnExpressions(context, func)) {
    const matched = expressionUsesCallee(expression, builder, context, func, new Set());
    if (matched === true) return true;
    if (matched === "unresolved") unresolved = true;
  }
  return unresolved ? "unresolved" : false;
}

function isConfiguredBuilder(func, builders, context) {
  const name = functionName(func);
  let unresolved = false;
  for (const builder of builders) {
    if (name && builder.names.includes(name)) return true;
    if (builder.callees.length === 0) continue;
    const matched = returnsConfiguredCallee(func, builder, context);
    if (matched === true) return true;
    if (matched === "unresolved") unresolved = true;
  }
  return unresolved ? "unresolved" : false;
}

function unwrapTagCallee(node) {
  let current = unwrapExpression(node);
  while (current?.type === "ParenthesizedExpression")
    current = unwrapExpression(current.expression);
  return current;
}

function identifierDeclaredOutside(identifier, context, func) {
  const variable = resolveVariable(identifier, context);
  if (!variable || variable.defs.length === 0) return true;
  return variable.defs.every((definition) => definition.node && !isInside(definition.node, func));
}

// A parameter or local object (`id.sql`, `id.x.sql`) throws once async is gone.
// A bare tag, or a member chain rooted outside the function, does not.
function tagCalleeIsStable(tag, context, func) {
  let current = unwrapTagCallee(tag);
  if (current?.type === "Identifier") return true;
  while (current?.type === "MemberExpression") {
    // A string or expression-free template key is the same member as `db.sql`.
    // A computed identifier can name a different property, so it stays unstable.
    if (current.computed && !staticKeyName(current)) return false;
    current = unwrapTagCallee(current.object);
  }
  return current?.type === "Identifier" && identifierDeclaredOutside(current, context, func);
}

module.exports = {
  calleeMatches,
  configuredBuilders,
  isConfiguredBuilder,
  localInitializer,
  tagCalleeIsStable,
};
