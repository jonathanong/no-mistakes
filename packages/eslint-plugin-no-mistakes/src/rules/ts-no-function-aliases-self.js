"use strict";

const {
  unwrapExpression,
  identifierName,
  staticMemberName,
} = require("./ts-no-function-aliases-ast");

function resolveVariable(sourceCode, identifier) {
  for (let scope = sourceCode.getScope(identifier); scope; scope = scope.upper) {
    const variable = scope.set.get(identifier.name);
    if (variable) return variable;
  }
  return null;
}

function sameIdentifier(left, right, sourceCode) {
  return (
    identifierName(left) !== null &&
    identifierName(left) === identifierName(right) &&
    resolveVariable(sourceCode, left) === resolveVariable(sourceCode, right)
  );
}

function sameStaticMember(left, right, sourceCode, allowThis) {
  const leftMember = unwrapExpression(left);
  const rightMember = unwrapExpression(right);
  if (
    staticMemberName(leftMember) === null ||
    staticMemberName(leftMember) !== staticMemberName(rightMember)
  ) {
    return false;
  }
  return sameReference(leftMember.object, rightMember.object, sourceCode, allowThis);
}

function sameReference(left, right, sourceCode, allowThis) {
  const lhs = unwrapExpression(left);
  const rhs = unwrapExpression(right);
  if (lhs.type !== rhs.type) return false;
  if (lhs.type === "Identifier") return sameIdentifier(lhs, rhs, sourceCode);
  if (lhs.type === "ThisExpression") return allowThis;
  if (lhs.type === "MemberExpression") return sameStaticMember(lhs, rhs, sourceCode, allowThis);
  return false;
}

function wrapperBindingIsFunction(node, callee, sourceCode) {
  if ((node.params || []).some((parameter) => patternHasName(parameter, callee.name))) return false;
  const variable = resolveVariable(sourceCode, callee);
  if (!variable) return false;
  return (
    variable.defs.length === 1 &&
    variable.defs.some((definition) => {
      const declaration = definition.node;
      return (
        declaration === node ||
        declaration?.init === node ||
        (declaration?.type === "VariableDeclarator" && declaration.init === node)
      );
    })
  );
}

function patternHasName(pattern, name) {
  if (!pattern) return false;
  if (pattern.type === "Identifier") return pattern.name === name;
  if (pattern.type === "RestElement") return patternHasName(pattern.argument, name);
  if (pattern.type === "AssignmentPattern") return patternHasName(pattern.left, name);
  if (pattern.type === "ArrayPattern")
    return pattern.elements.some((element) => patternHasName(element, name));
  if (pattern.type === "ObjectPattern") {
    return pattern.properties.some((property) =>
      patternHasName(property.type === "RestElement" ? property.argument : property.value, name),
    );
  }
  return false;
}

function assignmentTarget(node) {
  if (node.parent?.type !== "AssignmentExpression" || node.parent.right !== node) return null;
  return node.parent.left;
}

function assignmentWrapperName(node) {
  const left = assignmentTarget(node);
  if (left?.type === "Identifier") return left.name;
  if (left?.type === "MemberExpression") return staticMemberName(left);
  return null;
}

function variableWrapperName(node) {
  return node.parent?.type === "VariableDeclarator" && node.parent.id.type === "Identifier"
    ? node.parent.id.name
    : null;
}

function wrapperName(node) {
  return variableWrapperName(node) || assignmentWrapperName(node);
}

function isNamedWrapper(node) {
  if (node.type === "FunctionDeclaration") return true;
  return Boolean(wrapperName(node));
}

function isSelfTarget(node, target, sourceCode) {
  if (target?.type === "Identifier") return wrapperBindingIsFunction(node, target, sourceCode);
  const left = assignmentTarget(node);
  if (left?.type !== "MemberExpression" || target?.type !== "MemberExpression") return false;
  return sameStaticMember(left, target, sourceCode, node.type === "ArrowFunctionExpression");
}

function isSelfCall(node, call, sourceCode, forwarded) {
  const callee = unwrapExpression(call.callee);
  if (callee?.type === "Identifier") return isSelfTarget(node, callee, sourceCode);
  const left = assignmentTarget(node);
  if (
    left?.type === "MemberExpression" &&
    callee?.type === "MemberExpression" &&
    sameStaticMember(left, callee, sourceCode, node.type === "ArrowFunctionExpression")
  ) {
    return true;
  }
  return Boolean(forwarded && isSelfTarget(node, forwarded.target, sourceCode));
}

module.exports = {
  resolveVariable,
  sameReference,
  sameStaticMember,
  wrapperName,
  isNamedWrapper,
  isSelfCall,
  patternHasName,
};
