"use strict";

function primitiveType(type, ts, allowUndefined) {
  if (type.isUnion())
    return (
      type.types.length > 0 && type.types.every((part) => primitiveType(part, ts, allowUndefined))
    );
  const mask =
    ts.TypeFlags.StringLike |
    ts.TypeFlags.NumberLike |
    ts.TypeFlags.BooleanLike |
    ts.TypeFlags.BigIntLike |
    ts.TypeFlags.Null |
    (allowUndefined ? ts.TypeFlags.Undefined : 0);
  return Boolean(type.flags & mask);
}

function tsScalarDefault(node, ts) {
  return Boolean(
    node &&
    (ts.isStringLiteral(node) ||
      ts.isNumericLiteral(node) ||
      ts.isNoSubstitutionTemplateLiteral(node) ||
      [ts.SyntaxKind.TrueKeyword, ts.SyntaxKind.FalseKeyword, ts.SyntaxKind.NullKeyword].includes(
        node.kind,
      )),
  );
}

module.exports = { primitiveType, tsScalarDefault };
