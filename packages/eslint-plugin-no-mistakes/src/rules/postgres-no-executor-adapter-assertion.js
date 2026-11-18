"use strict";

const { safeOtherArguments } = require("./postgres-no-executor-adapter-nonwriting-arguments");
const { sourceWrites } = require("./postgres-no-executor-adapter-source");

function writtenNamespaces(writes, checker, ts, facts) {
  const cache = (facts.writtenAssertionNamespaces ??= new WeakMap());
  if (cache.has(writes)) return cache.get(writes);
  const modules = new Set();
  for (const symbol of writes.symbols) {
    if (
      symbol.flags & ts.SymbolFlags.Alias &&
      symbol.declarations?.length === 1 &&
      ts.isNamespaceImport(symbol.declarations[0])
    )
      modules.add(checker.getImmediateAliasedSymbol(symbol));
  }
  cache.set(writes, modules);
  return modules;
}

function stableTarget(symbol, callerWrites, checker, ts, facts, program) {
  const seen = new Set();
  while (symbol?.declarations?.length === 1 && !seen.has(symbol)) {
    seen.add(symbol);
    const declaration = symbol.declarations[0];
    if (ts.isTypeOnlyImportOrExportDeclaration(declaration)) return null;
    const source = declaration.getSourceFile();
    const writes = sourceWrites(source, checker, facts, program);
    const module = checker.getSymbolAtLocation(source);
    if (
      writes.eval ||
      writes.symbols.has(symbol) ||
      writes.declarations.has(declaration) ||
      callerWrites.symbols.has(symbol) ||
      callerWrites.declarations.has(declaration) ||
      (module &&
        (writtenNamespaces(callerWrites, checker, ts, facts).has(module) ||
          writtenNamespaces(writes, checker, ts, facts).has(module)))
    )
      return null;
    if (!(symbol.flags & ts.SymbolFlags.Alias)) return declaration;
    symbol = checker.getImmediateAliasedSymbol(symbol);
  }
  return null;
}

function stableCallSignature(call, checker, ts, facts, program) {
  const callee = call.expression;
  if (
    !ts.isIdentifier(callee) ||
    call.questionDotToken ||
    call.arguments.some((value) => ts.isSpreadElement(value))
  )
    return null;
  const callerWrites = sourceWrites(call.getSourceFile(), checker, facts, program);
  if (callerWrites.eval) return null;

  const type = checker.getTypeAtLocation(callee);
  if (type.flags & (ts.TypeFlags.Any | ts.TypeFlags.Unknown | ts.TypeFlags.Union)) return null;
  const signatures = checker.getSignaturesOfType(type, ts.SignatureKind.Call);
  if (signatures.length !== 1) return null;
  const signature = checker.getResolvedSignature(call);
  const declaration = signature?.getDeclaration();
  if (
    !declaration ||
    signatures[0].getDeclaration() !== declaration ||
    !ts.isFunctionDeclaration(declaration) ||
    declaration.asteriskToken ||
    declaration.modifiers?.some((modifier) => modifier.kind === ts.SyntaxKind.AsyncKeyword)
  )
    return null;
  const symbol = checker.getSymbolAtLocation(callee);
  return stableTarget(symbol, callerWrites, checker, ts, facts, program) === declaration
    ? signature
    : null;
}

function nonWritingAssertionBody(declaration, checker, ts, facts, program) {
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

  function literal(node) {
    return (
      ts.isStringLiteral(node) ||
      ts.isNumericLiteral(node) ||
      node.kind === ts.SyntaxKind.TrueKeyword ||
      node.kind === ts.SyntaxKind.FalseKeyword ||
      node.kind === ts.SyntaxKind.NullKeyword
    );
  }

  function guard(node) {
    if (literal(node)) return true;
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

  function thrownValue(node) {
    if (literal(node)) return true;
    if (
      !ts.isNewExpression(node) ||
      !ts.isIdentifier(node.expression) ||
      node.expression.text !== "Error" ||
      !node.arguments?.every(literal)
    )
      return false;
    const symbol = checker.getSymbolAtLocation(node.expression);
    return Boolean(
      symbol?.declarations?.length &&
      !writes.symbols.has(symbol) &&
      symbol.declarations.every((entry) =>
        program.isSourceFileDefaultLibrary(entry.getSourceFile()),
      ),
    );
  }

  function statement(node) {
    if (ts.isBlock(node)) return node.statements.every(statement);
    if (ts.isIfStatement(node))
      return (
        guard(node.expression) &&
        statement(node.thenStatement) &&
        (!node.elseStatement || statement(node.elseStatement))
      );
    if (ts.isThrowStatement(node)) return thrownValue(node.expression);
    if (ts.isReturnStatement(node)) return !node.expression;
    return ts.isEmptyStatement(node);
  }

  return statement(declaration.body);
}

function tsAssertionArgument(call, index, checker, ts, facts, program) {
  const signature = stableCallSignature(call, checker, ts, facts, program);
  if (!signature) return false;
  const predicate = checker.getTypePredicateOfSignature(signature);
  if (
    predicate?.kind !== ts.TypePredicateKind.AssertsIdentifier ||
    predicate.parameterIndex !== index ||
    !safeOtherArguments(call, index, checker, ts, facts, program)
  )
    return false;
  const declaration = signature.getDeclaration();
  const bodies = (facts.nonWritingAssertionBodies ??= new WeakMap());
  if (!bodies.has(declaration))
    bodies.set(declaration, nonWritingAssertionBody(declaration, checker, ts, facts, program));
  return bodies.get(declaration);
}

module.exports = { stableCallSignature, tsAssertionArgument };
