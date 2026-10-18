"use strict";

const { findContainingFunction } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { propertyName } = require("./module-mock-helpers");
const { annotationContract, combine } = require("./postgres-no-async-sql-builder-types");

const PROMISE_USES = new Set(["then", "catch", "finally"]);

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

function isThenObject(node) {
  const member = node.parent;
  if (member?.type !== "MemberExpression" || member.object !== node || member.computed)
    return false;
  const name = propertyName(member.property);
  return (
    PROMISE_USES.has(name) &&
    member.parent?.type === "CallExpression" &&
    member.parent.callee === member
  );
}

function isThenCall(node) {
  return isThenObject(node);
}

function bindingResultContract(declarator, context, types) {
  let result = annotationContract(declarator.id.typeAnnotation, types);
  if (declarationExported(declarator.parent)) result = combine([result, "unknown"]);
  const variable = resolveVariable(declarator.id, context);
  if (!variable || hasLaterWrite(variable)) return combine([result, "unknown"]);
  for (const reference of variable.references) {
    if (reference.init) continue;
    if (isThenObject(reference.identifier)) return "requires";
    const parent = reference.identifier.parent;
    if (parent?.type === "MemberExpression" && parent.object === reference.identifier)
      return combine([result, "unknown"]);
  }
  return result;
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
  isThenCall,
  returnedPromiseContract,
};
