"use strict";

const { unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");

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
  if (variable.references.some((reference) => reference.isWrite() && !reference.init)) return null;
  seen.add(variable);
  return unwrapExpression(binding.init);
}

function isModuleConstSql(identifier, context, seen = new Set()) {
  const init = moduleConstInit(identifier, context, seen);
  if (!init) return false;
  if (literalSql(init)) return true;
  if (init.type !== "Identifier") return false;
  return isModuleConstSql(init, context, seen);
}

module.exports = {
  isModuleConstSql,
};
