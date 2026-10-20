"use strict";

const { unwrapExpression } = require("./async-ast");
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
  if (literalSql(init)) return variable.defs[0].parent;
  if (init.type !== "Identifier" || !moduleConstSqlDeclaration(init, context, seen)) return null;
  // The call reads the outer binding. Its initializer runs only after that
  // declaration, so the cutoff is this declaration, not the chain.
  return variable.defs[0].parent;
}

function isModuleConstSql(identifier, context, seen = new Set()) {
  return moduleConstSqlDeclaration(identifier, context, seen) !== null;
}

// A hoisted call before the const throws once `async` is gone, instead of
// rejecting a promise and letting the rest of the module run.
function moduleConstsAreReady(func, declarations, context) {
  if (declarations.length === 0) return true;
  const variable = boundVariable(func, context);
  if (!variable) return false;
  const cutoff = Math.max(...declarations.map((declaration) => declaration.range[1]));
  return variable.references.every((reference) => {
    if (reference.init || isInside(reference.identifier, func)) return true;
    return reference.identifier.range[0] >= cutoff;
  });
}

module.exports = {
  isModuleConstSql,
  moduleConstSqlDeclaration,
  moduleConstsAreReady,
};
