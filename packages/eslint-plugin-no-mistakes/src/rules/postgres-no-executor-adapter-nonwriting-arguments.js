"use strict";

const { sourceWrites } = require("./postgres-no-executor-adapter-source");
const { primitiveType } = require("./postgres-no-executor-adapter-sql-primitive");

const unaryOperators = new Set(["!", "+", "-", "~"]);
const maxDepth = 8;

function safeOtherArguments(call, selectedIndex, checker, ts, facts, program) {
  const source = call.getSourceFile();

  function stablePrimitive(node, depth) {
    const symbol = checker.getSymbolAtLocation(node);
    if (symbol?.declarations?.length !== 1) return false;
    const declaration = symbol.declarations[0];
    if (declaration.getSourceFile() !== source) return false;
    const writes = sourceWrites(source, checker, facts, program);
    if (writes.eval || writes.symbols.has(symbol)) return false;
    if (ts.isParameter(declaration))
      return Boolean(
        ts.isIdentifier(declaration.name) &&
        declaration.type &&
        primitiveType(checker.getTypeFromTypeNode(declaration.type), ts, true) &&
        primitiveType(checker.getTypeAtLocation(node), ts, true),
      );
    if (
      !ts.isVariableDeclaration(declaration) ||
      !ts.isVariableDeclarationList(declaration.parent) ||
      (declaration.parent.flags & ts.NodeFlags.BlockScoped) !== ts.NodeFlags.Const ||
      !declaration.initializer
    )
      return false;
    return scalar(declaration.initializer, depth);
  }

  function scalar(node, depth) {
    if (depth === 0) return false;
    if (
      ts.isStringLiteral(node) ||
      ts.isNoSubstitutionTemplateLiteral(node) ||
      ts.isNumericLiteral(node) ||
      ts.isBigIntLiteral(node) ||
      node.kind === ts.SyntaxKind.TrueKeyword ||
      node.kind === ts.SyntaxKind.FalseKeyword ||
      node.kind === ts.SyntaxKind.NullKeyword
    )
      return true;
    if (ts.isIdentifier(node)) return stablePrimitive(node, depth - 1);
    if (
      ts.isParenthesizedExpression(node) ||
      ts.isAsExpression(node) ||
      ts.isTypeAssertionExpression(node) ||
      ts.isNonNullExpression(node) ||
      ts.isSatisfiesExpression(node) ||
      ts.isVoidExpression(node) ||
      ts.isTypeOfExpression(node)
    )
      return scalar(node.expression, depth - 1);
    return (
      ts.isPrefixUnaryExpression(node) &&
      unaryOperators.has(ts.tokenToString(node.operator)) &&
      scalar(node.operand, depth - 1)
    );
  }

  return call.arguments.every(
    (argument, index) => index === selectedIndex || scalar(argument, maxDepth),
  );
}

module.exports = { safeOtherArguments };
