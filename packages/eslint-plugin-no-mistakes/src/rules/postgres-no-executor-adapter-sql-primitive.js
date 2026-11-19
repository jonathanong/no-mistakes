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

// Local hole checks accept a primitive `for...of` binding. The imported path uses this
// so a primitive loop variable is not rejected solely for being a loop binding.
function tsForOfUnproven(source, node, field, checker, ts, program) {
  if (!constForOf(source, ts)) return false;
  if (field !== null) return true;
  const { bindValueType } = require("./postgres-no-executor-adapter-sql-fresh");
  return !bindValueType(checker.getTypeAtLocation(node), ts, program, true);
}

function constForOf(source, ts) {
  const list = source.parent;
  return Boolean(
    ts.isVariableDeclaration(source) &&
    ts.isVariableDeclarationList(list) &&
    list.flags & ts.NodeFlags.Const &&
    ts.isForOfStatement(list.parent),
  );
}

// A const for-of binding is recorded as a write by the loop itself. A primitive one is still a scalar.
function constPrimitiveForOf(source, node, field, checker, ts, program) {
  return constForOf(source, ts) && !tsForOfUnproven(source, node, field, checker, ts, program);
}

function constPrimitiveForOfSet(declarations, node, field, checker, ts, program) {
  return Boolean(
    declarations?.length &&
    declarations.every((source) => constPrimitiveForOf(source, node, field, checker, ts, program)),
  );
}

module.exports = {
  constPrimitiveForOfSet,
  primitiveType,
  tsForOfUnproven,
  tsScalarDefault,
};
