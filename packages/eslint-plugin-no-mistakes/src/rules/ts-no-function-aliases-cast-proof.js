"use strict";

const { staticMemberName } = require("./ts-no-function-aliases-ast");
const { resolveVariable } = require("./ts-no-function-aliases-self");

function unassertedPath(expression) {
  let current = expression;
  while (["ChainExpression", "MemberExpression", "AwaitExpression"].includes(current.type)) {
    if (current.type === "ChainExpression") current = current.expression;
    else current = current.type === "AwaitExpression" ? current.argument : current.object;
  }
  return !current.type.startsWith("TS");
}

function typeQueryPath(node) {
  if (node?.type === "Identifier") return [node.name];
  if (node?.type === "TSQualifiedName") {
    const left = typeQueryPath(node.left);
    return left && [...left, node.right.name];
  }
  return null;
}

function pathRoot(node) {
  let current = node;
  while (current?.type === "MemberExpression" || current?.type === "TSQualifiedName")
    current = current.type === "MemberExpression" ? current.object : current.left;
  return current?.type === "Identifier" ? current : null;
}

function valuePath(node) {
  if (node?.type === "Identifier") return [node.name];
  if (node?.type === "MemberExpression") {
    const left = valuePath(node.object);
    const right = staticMemberName(node);
    return left && right !== null ? [...left, right] : null;
  }
  return null;
}

function samePath(left, right) {
  return (
    left &&
    right &&
    left.length === right.length &&
    left.every((segment, index) => segment === right[index])
  );
}

function assertionType(annotation, sourceCode) {
  if (
    annotation.type !== "TSTypeReference" ||
    annotation.typeName.type !== "Identifier" ||
    annotation.typeArguments
  )
    return annotation;
  const variable = resolveVariable(sourceCode, annotation.typeName);
  const declaration = variable?.defs.length === 1 ? variable.defs[0].node : null;
  if (declaration?.type !== "TSTypeAliasDeclaration" || declaration.typeParameters)
    return annotation;
  // One non-generic, same-file alias is a local proof. Chained aliases and
  // imported type names need type resolution that this rule does not own.
  return declaration.typeAnnotation.type === "TSTypeReference"
    ? annotation
    : declaration.typeAnnotation;
}

function preservedCalleeCast(node, target, sourceCode) {
  if (node.type !== "TSAsExpression" && node.type !== "TSTypeAssertion") return false;
  const annotation = assertionType(node.typeAnnotation, sourceCode);
  return (
    samePath(valuePath(node.expression), valuePath(target)) &&
    annotation.type === "TSTypeQuery" &&
    samePath(typeQueryPath(annotation.exprName), valuePath(target)) &&
    resolveVariable(sourceCode, pathRoot(annotation.exprName)) ===
      resolveVariable(sourceCode, pathRoot(target)) &&
    !annotation.typeArguments
  );
}

function resultCastPreserved(node, target, sourceCode) {
  const castType = assertionType(node.typeAnnotation, sourceCode).type;
  return (
    (node.type === "TSAsExpression" || node.type === "TSTypeAssertion") &&
    target.callable &&
    !wrapperHasOverloads(target.callable, sourceCode) &&
    !target.callable.typeParameters &&
    !target.callable.async &&
    !target.callable.generator &&
    target.callable?.returnType?.typeAnnotation.type === castType &&
    [
      "TSVoidKeyword",
      "TSUndefinedKeyword",
      "TSNullKeyword",
      "TSStringKeyword",
      "TSNumberKeyword",
      "TSBooleanKeyword",
      "TSBigIntKeyword",
      "TSSymbolKeyword",
    ].includes(castType)
  );
}

function assertedContractPreserved(node, call, candidate, target, sourceCode) {
  let result = node.body;
  if (result.type === "BlockStatement") {
    const statement = result.body[0];
    result = statement.argument || statement.expression;
  }
  while (result !== call) {
    if (!result) return false;
    if (["TSAsExpression", "TSTypeAssertion"].includes(result.type)) {
      if (!resultCastPreserved(result, target, sourceCode)) return false;
      result = result.expression;
    } else if (result.type === "ChainExpression" || result.type === "TSNonNullExpression") {
      if (result.type === "TSNonNullExpression") return false;
      result = result.expression;
    } else if (result.type === "AwaitExpression") {
      result = result.argument;
    } else return false;
  }
  let callee = call.callee;
  while (callee && callee !== candidate.target) {
    if (["TSAsExpression", "TSTypeAssertion"].includes(callee.type)) {
      if (!preservedCalleeCast(callee, candidate.target, sourceCode)) return false;
      callee = callee.expression;
    } else if (callee.type === "ChainExpression") {
      callee = callee.expression;
    } else if (callee.type === "MemberExpression") {
      callee = callee.object;
    } else return false;
  }
  return callee === candidate.target && unassertedPath(candidate.target);
}

function wrapperHasOverloads(node, sourceCode) {
  if (node.type !== "FunctionDeclaration") return false;
  return sourceCode
    .getDeclaredVariables(node)
    .some((variable) =>
      variable.defs.some(
        (definition) => definition.type === "FunctionName" && definition.node !== node,
      ),
    );
}

module.exports = { unassertedPath, assertedContractPreserved, wrapperHasOverloads };
