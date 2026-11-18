"use strict";

const { constInitializer } = require("./postgres-no-executor-adapter-binding");
const { typedScalarProjection } = require("./postgres-no-executor-adapter-sql-type");
const { scalarArgument } = require("./postgres-no-executor-adapter-sql-scalar");
const { resolveVariable, unwrapTs } = require("./postgres-query-text");

function localField(pattern, variable, context) {
  if (pattern?.type !== "ObjectPattern") return null;
  for (const entry of pattern.properties) {
    if (entry.type !== "Property" || entry.computed || entry.value.type !== "Identifier") continue;
    const key = entry.key.type === "Identifier" ? entry.key.name : entry.key.value;
    if (typeof key === "string" && resolveVariable(entry.value, context) === variable) return key;
  }
  return null;
}

function tsField(pattern, symbol, checker, ts) {
  if (!ts.isObjectBindingPattern(pattern)) return null;
  for (const entry of pattern.elements) {
    if (!ts.isIdentifier(entry.name) || entry.dotDotDotToken || entry.initializer) continue;
    const key = entry.propertyName ?? entry.name;
    if (
      (ts.isIdentifier(key) || ts.isStringLiteral(key)) &&
      checker.getSymbolAtLocation(entry.name) === symbol
    )
      return key.text;
  }
  return null;
}

function localAliasField(def, variable, context) {
  if (!def.node.init) return null;
  const field = localField(def.node.id, variable, context);
  return field === null ? null : { initializer: def.node.init, field };
}

function tsAliasField(source, symbol, checker, ts) {
  const declaration = source.parent.parent;
  if (!ts.isVariableDeclaration(declaration) || !declaration.initializer) return null;
  const field = tsField(source.parent, symbol, checker, ts);
  return field === null ? null : { initializer: declaration.initializer, field };
}

function tsUnwrap(node, wrappers) {
  while (node && wrappers.has(node.kind)) node = node.expression;
  return node;
}

function tsObjectAliased(declaration, symbol, checker, ts, facts, executorSymbol, allowedProperty) {
  const cache = (facts.projectionAliases ??= new WeakMap());
  let byReceiver = cache.get(declaration);
  if (!byReceiver) {
    byReceiver = new Map();
    cache.set(declaration, byReceiver);
  }
  let byProperty = byReceiver.get(executorSymbol);
  if (!byProperty) {
    byProperty = new Map();
    byReceiver.set(executorSymbol, byProperty);
  }
  let aliases = byProperty.get(allowedProperty);
  if (!aliases) {
    aliases = new Set();
    const wrappers = new Set([
      ts.SyntaxKind.AsExpression,
      ts.SyntaxKind.TypeAssertionExpression,
      ts.SyntaxKind.NonNullExpression,
      ts.SyntaxKind.SatisfiesExpression,
      ts.SyntaxKind.ParenthesizedExpression,
    ]);
    function visit(node) {
      if (ts.isCallExpression(node))
        for (const argument of node.arguments)
          if (ts.isIdentifier(tsUnwrap(argument, wrappers)))
            aliases.add(checker.getSymbolAtLocation(tsUnwrap(argument, wrappers)));
      if (
        ts.isCallExpression(node) &&
        ts.isPropertyAccessExpression(tsUnwrap(node.expression, wrappers))
      ) {
        const member = tsUnwrap(node.expression, wrappers);
        if (ts.isIdentifier(member.expression)) {
          const receiver = checker.getSymbolAtLocation(member.expression);
          if (receiver !== executorSymbol || member.name.text !== allowedProperty)
            aliases.add(receiver);
        }
      }
      if (
        ts.isVariableDeclaration(node) &&
        ts.isIdentifier(node.name) &&
        node.initializer &&
        ts.isIdentifier(tsUnwrap(node.initializer, wrappers))
      )
        aliases.add(checker.getSymbolAtLocation(tsUnwrap(node.initializer, wrappers)));
      if (ts.isShorthandPropertyAssignment(node))
        aliases.add(checker.getShorthandAssignmentValueSymbol(node));
      if (ts.isPropertyAssignment(node) && ts.isIdentifier(tsUnwrap(node.initializer, wrappers)))
        aliases.add(checker.getSymbolAtLocation(tsUnwrap(node.initializer, wrappers)));
      ts.forEachChild(node, visit);
    }
    visit(declaration.body);
    byProperty.set(allowedProperty, aliases);
  }
  return aliases.has(symbol);
}

function staticObjectValue(node, field, context, seen = new Set()) {
  node = unwrapTs(node);
  if (node.type === "Identifier") {
    const variable = resolveVariable(node, context);
    if (variable?.references.some((reference) => !reference.init && reference.identifier !== node))
      return null;
    const init = constInitializer(node, context, seen);
    return init ? staticObjectValue(init, field, context, seen) : null;
  }
  if (node.type !== "ObjectExpression") return null;
  let value = null;
  for (const entry of node.properties) {
    if (entry.type !== "Property" || entry.computed || entry.kind !== "init") return null;
    const key = entry.key.type === "Identifier" ? entry.key.name : entry.key.value;
    if (key === field) {
      if (value) return null;
      value = entry.value;
    }
  }
  return value;
}

function projectedScalar(node, field, context, allowUndefined) {
  if (field !== null && scalarArgument(node, context, allowUndefined)) return true;
  if (field !== null && typedScalarProjection(node, field, context, allowUndefined)) return true;
  const value = field === null ? node : staticObjectValue(node, field, context);
  return Boolean(value && scalarArgument(value, context, allowUndefined));
}

module.exports = {
  localAliasField,
  localField,
  projectedScalar,
  tsAliasField,
  tsField,
  tsObjectAliased,
};
