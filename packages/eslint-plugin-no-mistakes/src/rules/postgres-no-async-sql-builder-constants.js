"use strict";

const { findContainingFunction, unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const {
  boundVariable,
  declarationExported,
  isInside,
} = require("./postgres-no-async-sql-builder-bindings");

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

function hoistedExport(func, variable) {
  // `export function` is callable while dependencies evaluate, before any
  // const initializer. A missing local call is not proof of order.
  if (func.type !== "FunctionDeclaration") return false;
  if (declarationExported(func)) return true;
  return variable.references.some(
    (reference) => reference.identifier.parent?.type === "ExportSpecifier",
  );
}

function classOwner(func) {
  const member = func.parent;
  if (
    (member?.type !== "MethodDefinition" && member?.type !== "PropertyDefinition") ||
    member.value !== func
  ) {
    return null;
  }
  let owner = member.parent;
  while (owner && owner.type !== "ClassDeclaration" && owner.type !== "ClassExpression") {
    owner = owner.parent;
  }
  return owner;
}

function classVariable(owner, context) {
  const parent = owner.parent;
  // `const A = class {}` is the const binding. A class declaration's name also
  // exists inside the class, and that inner binding has no outside calls.
  if (
    parent?.type === "VariableDeclarator" &&
    parent.init === owner &&
    parent.id.type === "Identifier"
  ) {
    return resolveVariable(parent.id, context);
  }
  if (owner.id?.type !== "Identifier") return null;
  // A class declaration's outer binding lives on the enclosing scope.
  return context.sourceCode
    .getScope(owner.id)
    .upper.variables.find((item) => item.name === owner.id.name);
}

function classIsExported(owner, variable) {
  if (declarationExported(owner)) return true;
  if (owner.parent?.type === "VariableDeclarator" && declarationExported(owner.parent.parent)) {
    return true;
  }
  return (
    variable?.references.some(
      (reference) => reference.identifier.parent?.type === "ExportSpecifier",
    ) ?? false
  );
}

// One binding's call sites. A cycle is not ordered: the const may still be
// in its temporal dead zone when the cycle runs.
function referencesRunAfter(owner, variable, cutoff, context, seen) {
  if (seen.has(variable)) return false;
  seen.add(variable);
  let ready = true;
  for (const reference of variable.references) {
    if (!referenceRunsAfter(reference, owner, cutoff, context, seen)) {
      ready = false;
      break;
    }
  }
  seen.delete(variable);
  return ready;
}

function classMemberRunsAfter(func, cutoff, context, seen) {
  const owner = classOwner(func);
  if (!owner) return true;
  const variable = classVariable(owner, context);
  // An export is not proof the method runs after the const.
  if (classIsExported(owner, variable)) return false;
  if (!variable) return true;
  return referencesRunAfter(func, variable, cutoff, context, seen);
}

function unboundRunsAfter(func, cutoff, context, seen) {
  // `export default function () {}` is hoisted and has no binding. A cyclic
  // import can call it before the const, so textual position is not readiness.
  if (func.type === "FunctionDeclaration" && func.parent.type === "ExportDefaultDeclaration") {
    return false;
  }
  // An arrow or function expression is not hoisted, so it cannot run before
  // its own start. A callback still runs whenever the enclosing function runs,
  // and a class method runs whenever that class is constructed.
  if (func.range[0] < cutoff) return false;
  const outer = findContainingFunction(func);
  if (outer) return functionRunsAfter(outer, cutoff, context, seen);
  return classMemberRunsAfter(func, cutoff, context, seen);
}

// `const init = () => {}` before the const is live as soon as that line runs.
// An export lets a cycle call it before a later const, wherever `export { init }` sits.
function exportRunsEarly(func, variable, cutoff) {
  const declarator = func.parent;
  if (declarator?.type !== "VariableDeclarator" || declarator.range[0] >= cutoff) return false;
  if (declarationExported(declarator.parent)) return true;
  return variable.references.some(
    (reference) => reference.identifier.parent?.type === "ExportSpecifier",
  );
}

function functionRunsAfter(func, cutoff, context, seen) {
  const variable = boundVariable(func, context);
  if (!variable) return unboundRunsAfter(func, cutoff, context, seen);
  if (hoistedExport(func, variable) || exportRunsEarly(func, variable, cutoff)) return false;
  return referencesRunAfter(func, variable, cutoff, context, seen);
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
