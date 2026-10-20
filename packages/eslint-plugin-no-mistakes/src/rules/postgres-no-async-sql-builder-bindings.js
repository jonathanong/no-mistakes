"use strict";

const { findContainingFunction } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { annotationContract, combine } = require("./postgres-no-async-sql-builder-types");

function declarationExported(node) {
  const parent = node?.parent;
  return parent?.type === "ExportNamedDeclaration" || parent?.type === "ExportDefaultDeclaration";
}

function hasLaterWrite(variable, ignored) {
  return variable.references.some(
    (reference) => reference.isWrite() && !reference.init && reference.identifier !== ignored,
  );
}

function boundVariable(func, context) {
  if (func.type === "FunctionDeclaration" && func.id) return resolveVariable(func.id, context);
  const parent = func.parent;
  if (
    parent?.type === "VariableDeclarator" &&
    parent.init === func &&
    parent.id.type === "Identifier"
  ) {
    return resolveVariable(parent.id, context);
  }
  if (
    parent?.type === "AssignmentExpression" &&
    parent.right === func &&
    parent.left.type === "Identifier"
  ) {
    return resolveVariable(parent.left, context);
  }
  return null;
}

function exportContract(func, context) {
  if (declarationExported(func)) return "unknown";
  const variable = boundVariable(func, context);
  if (
    variable?.references.some(
      (reference) => reference.identifier.parent?.type === "ExportSpecifier",
    )
  ) {
    return "unknown";
  }
  const declarator = func.parent?.type === "VariableDeclarator" ? func.parent : null;
  return declarator && declarationExported(declarator.parent) ? "unknown" : "allows";
}

function isInside(node, ancestor) {
  let current = node;
  while (current) {
    if (current === ancestor) return true;
    current = current.parent;
  }
  return false;
}

function bindingResultContract(declarator, context, types, seen = new Set()) {
  if (seen.has(declarator)) return "unknown";
  let result = annotationContract(declarator.id.typeAnnotation, types);
  if (declarationExported(declarator.parent)) result = combine([result, "unknown"]);
  const variable = resolveVariable(declarator.id, context);
  if (!variable || hasLaterWrite(variable)) return combine([result, "unknown"]);
  // calls.js imports this file. Load it here so each read uses the direct-call
  // contract: arguments, returns, aliases, and nothing else.
  const { callResultContract } = require("./postgres-no-async-sql-builder-calls");
  seen.add(declarator);
  const results = [result];
  for (const reference of variable.references) {
    if (reference.init) continue;
    results.push(callResultContract(reference.identifier, context, types, seen));
  }
  seen.delete(declarator);
  return combine(results);
}

function returnedPromiseContract(returnStatement, context, types) {
  const owner = findContainingFunction(returnStatement);
  if (owner.async) {
    return "allows";
  } else if (owner.returnType) {
    return annotationContract(owner.returnType, types);
  } else {
    return "unknown";
  }
}

module.exports = {
  bindingResultContract,
  boundVariable,
  declarationExported,
  exportContract,
  hasLaterWrite,
  isInside,
  returnedPromiseContract,
};
