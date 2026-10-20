"use strict";

const { findContainingFunction, unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { boundVariable, isInside } = require("./postgres-no-async-sql-builder-bindings");

// Module scope only folds string literals, no-substitution templates, and
// const bindings of those. Other module expressions are not judged SQL.
function literalSql(node) {
  if (node.type === "Literal") return typeof node.value === "string";
  return node.type === "TemplateLiteral" && node.expressions.length === 0;
}

function atModuleScope(declaration) {
  const statement = declaration.parent;
  return statement.type === "Program" || statement.type === "ExportNamedDeclaration";
}

function moduleConstInit(identifier, context, seen) {
  const variable = resolveVariable(identifier, context);
  if (!variable || seen.has(variable) || variable.defs.length !== 1) return null;
  const definition = variable.defs[0];
  const declaration = definition.parent;
  if (definition.type !== "Variable" || declaration.kind !== "const") return null;
  const binding = definition.node;
  if (binding.id.type !== "Identifier") return null;
  if (!atModuleScope(declaration)) return null;
  // `declare const Q: string` has no initializer. Do not unwrap null.
  if (!binding.init) return null;
  if (variable.references.some((reference) => reference.isWrite() && !reference.init)) return null;
  seen.add(variable);
  return unwrapExpression(binding.init);
}

function moduleConstSqlDeclaration(identifier, context, seen = new Set()) {
  const variable = resolveVariable(identifier, context);
  const init = moduleConstInit(identifier, context, seen);
  if (!variable || !init) return null;
  // Later declarators in the same statement run after this initializer.
  // The cutoff is this declarator, not the whole declaration or the chain.
  const declarator = variable.defs[0].node;
  if (literalSql(init)) return declarator;
  if (init.type !== "Identifier" || !moduleConstSqlDeclaration(init, context, seen)) return null;
  return declarator;
}

function isModuleConstSql(identifier, context, seen = new Set()) {
  return moduleConstSqlDeclaration(identifier, context, seen) !== null;
}

// A call before the const throws once `async` is gone. A reference nested in
// another function counts only when that function's own calls are after the
// const; otherwise the helper may already have run.
function referenceRunsAfter(reference, owner, cutoff, context, seen) {
  if (reference.init || isInside(reference.identifier, owner)) return true;
  const container = findContainingFunction(reference.identifier);
  if (!container || container === owner) return reference.identifier.range[0] >= cutoff;
  return functionRunsAfter(container, cutoff, context, seen);
}

function functionRunsAfter(func, cutoff, context, seen) {
  const variable = boundVariable(func, context);
  if (!variable || seen.has(variable)) return false;
  seen.add(variable);
  let ready = true;
  for (const reference of variable.references) {
    if (!referenceRunsAfter(reference, func, cutoff, context, seen)) {
      ready = false;
      break;
    }
  }
  seen.delete(variable);
  return ready;
}

function moduleConstsAreReady(func, declarations, context) {
  if (declarations.length === 0) return true;
  const cutoff = Math.max(...declarations.map((declaration) => declaration.range[1]));
  return functionRunsAfter(func, cutoff, context, new Set());
}

module.exports = {
  isModuleConstSql,
  moduleConstSqlDeclaration,
  moduleConstsAreReady,
};
