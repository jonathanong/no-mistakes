"use strict";

const { safeOtherArguments } = require("./postgres-no-executor-adapter-nonwriting-arguments");
const { sourceWrites } = require("./postgres-no-executor-adapter-source");
const {
  assertionThrownValue,
  literal,
} = require("./postgres-no-executor-adapter-assertion-thrown");

const {
  stableCallSignature,
  assertionDeclaration,
} = require("./postgres-no-executor-adapter-assertion-target");

function nonWritingAssertionBody(declaration, checker, ts, facts, program, intrinsics) {
  if (!declaration.body)
    return (
      declaration.getSourceFile().isDeclarationFile ||
      Boolean(ts.getCombinedModifierFlags(declaration) & ts.ModifierFlags.Ambient)
    );
  const parameters = new Set();
  for (const parameter of declaration.parameters) {
    if (!ts.isIdentifier(parameter.name) || parameter.initializer || parameter.dotDotDotToken)
      return false;
    const symbol = checker.getSymbolAtLocation(parameter.name);
    parameters.add(symbol);
  }
  const writes = sourceWrites(declaration.getSourceFile(), checker, facts, program);

  function guard(node) {
    if (literal(node, ts)) return true;
    if (ts.isIdentifier(node)) return parameters.has(checker.getSymbolAtLocation(node));
    if (ts.isParenthesizedExpression(node) || ts.isTypeOfExpression(node))
      return guard(node.expression);
    if (ts.isPrefixUnaryExpression(node))
      return node.operator === ts.SyntaxKind.ExclamationToken && guard(node.operand);
    if (ts.isBinaryExpression(node))
      return (
        [
          ts.SyntaxKind.EqualsEqualsEqualsToken,
          ts.SyntaxKind.ExclamationEqualsEqualsToken,
          ts.SyntaxKind.AmpersandAmpersandToken,
          ts.SyntaxKind.BarBarToken,
        ].includes(node.operatorToken.kind) &&
        guard(node.left) &&
        guard(node.right)
      );
    return false;
  }

  function statement(node) {
    if (ts.isBlock(node)) return node.statements.every(statement);
    if (ts.isIfStatement(node))
      return (
        guard(node.expression) &&
        statement(node.thenStatement) &&
        (!node.elseStatement || statement(node.elseStatement))
      );
    if (ts.isThrowStatement(node))
      return assertionThrownValue(
        node.expression,
        checker,
        ts,
        program,
        writes,
        parameters,
        intrinsics,
      );
    if (ts.isReturnStatement(node)) return !node.expression;
    return ts.isEmptyStatement(node);
  }

  return statement(declaration.body);
}

function tsAssertionArgument(call, index, checker, ts, facts, program) {
  const signature = stableCallSignature(call, checker, ts, facts, program, index);
  if (!signature) return false;
  const predicate = checker.getTypePredicateOfSignature(signature);
  if (
    predicate?.kind !== ts.TypePredicateKind.AssertsIdentifier ||
    predicate.parameterIndex !== index ||
    !safeOtherArguments(call, index, checker, ts, facts, program)
  )
    return false;
  const declaration = assertionDeclaration(signature, checker, ts);
  const bodies = (facts.nonWritingAssertionBodies ??= new WeakMap());
  if (!bodies.has(declaration)) {
    const intrinsics = new Set();
    const safe = nonWritingAssertionBody(declaration, checker, ts, facts, program, intrinsics);
    bodies.set(declaration, { safe, intrinsics });
  }
  const proof = bodies.get(declaration);
  const writes = sourceWrites(call.getSourceFile(), checker, facts, program);
  return (
    proof.safe &&
    [...proof.intrinsics].every(
      (symbol) =>
        !writes.symbols.has(symbol) &&
        !writes.escapedSymbols.has(symbol) &&
        symbol.declarations.every((entry) => !writes.declarations.has(entry)),
    )
  );
}

module.exports = { stableCallSignature, tsAssertionArgument };
