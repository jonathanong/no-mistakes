"use strict";

// Identical primitive keywords have file-local meaning. Identical spelling of
// a type reference, predicate, generic or structural type is not a type proof.
const PRIMITIVES = new Set([
  "TSAnyKeyword",
  "TSUnknownKeyword",
  "TSNeverKeyword",
  "TSVoidKeyword",
  "TSUndefinedKeyword",
  "TSNullKeyword",
  "TSStringKeyword",
  "TSNumberKeyword",
  "TSBooleanKeyword",
  "TSBigIntKeyword",
  "TSSymbolKeyword",
  "TSObjectKeyword",
]);

function identicalType(left, right) {
  const type = left?.typeAnnotation;
  return Boolean(type && PRIMITIVES.has(type.type) && type.type === right?.typeAnnotation?.type);
}

function hasDeclaredSignature(node) {
  return Boolean(
    node.typeParameters ||
    node.returnType ||
    node.params.some((param) => param.typeAnnotation || param.optional),
  );
}

function signaturePreserved(node, target) {
  if (node.async || node.generator) return false;
  const dropsResult =
    node.body.type === "BlockStatement" && node.body.body[0].type === "ExpressionStatement";
  if (dropsResult && target.callable?.returnType?.typeAnnotation.type !== "TSVoidKeyword")
    return false;
  const parent = node.parent;
  // An assignment can acquire a contextual property/call signature that is not
  // declared on this function. An annotated binding does the same.
  if (parent?.type === "AssignmentExpression" || parent?.id?.typeAnnotation) return false;
  if (!hasDeclaredSignature(node)) return true;
  const original = target.callable;
  if (
    target.annotation ||
    !original?.params ||
    original.typeParameters ||
    node.typeParameters ||
    original.params.length !== node.params.length
  )
    return false;
  if (node.returnType && !identicalType(node.returnType, original.returnType)) return false;
  return node.params.every((param, index) => {
    const other = original.params[index];
    return (
      other.type === param.type &&
      param.optional === other.optional &&
      (!param.typeAnnotation || identicalType(param.typeAnnotation, other.typeAnnotation))
    );
  });
}

module.exports = { signaturePreserved };
