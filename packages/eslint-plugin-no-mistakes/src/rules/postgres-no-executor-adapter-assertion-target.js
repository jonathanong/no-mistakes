"use strict";

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
  while (symbol?.declarations?.length && !seen.has(symbol)) {
    seen.add(symbol);
    if (
      checker.getTypeOnlyAliasDeclaration(symbol) ||
      (symbol.declarations.length > 1 &&
        (!symbol.declarations.some(ts.isFunctionDeclaration) ||
          !symbol.declarations.every(
            (entry) =>
              entry.getSourceFile() === symbol.declarations[0].getSourceFile() &&
              (ts.isFunctionDeclaration(entry) ||
                (ts.isModuleDeclaration(entry) &&
                  (entry.getSourceFile().isDeclarationFile ||
                    Boolean(ts.getCombinedModifierFlags(entry) & ts.ModifierFlags.Ambient)))),
          )))
    )
      return null;
    for (const declaration of symbol.declarations) {
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
    }
    if (!(symbol.flags & ts.SymbolFlags.Alias)) return symbol;
    symbol = checker.getImmediateAliasedSymbol(symbol);
  }
  return null;
}

function variableOwner(declaration, ts) {
  let node = declaration;
  while (node && !ts.isSourceFile(node)) {
    if (ts.isVariableDeclaration(node)) return node;
    node = node.parent;
  }
  return null;
}

function assertionDeclaration(signature, checker, ts) {
  const declaration = signature.getDeclaration();
  if (ts.isFunctionDeclaration(declaration)) {
    const symbol = declaration.name && checker.getSymbolAtLocation(declaration.name);
    return (
      symbol?.declarations?.find((entry) => ts.isFunctionDeclaration(entry) && entry.body) ??
      declaration
    );
  }
  const variable = variableOwner(declaration, ts);
  return variable?.initializer ?? declaration;
}

function callableVariable(declaration, ts) {
  return (
    ts.isVariableDeclaration(declaration) &&
    ts.isIdentifier(declaration.name) &&
    ts.isVariableDeclarationList(declaration.parent) &&
    (declaration.parent.flags & ts.NodeFlags.BlockScoped) === ts.NodeFlags.Const &&
    declaration.initializer &&
    (ts.isArrowFunction(declaration.initializer) ||
      ts.isFunctionExpression(declaration.initializer))
  );
}

function stableCallSignature(call, checker, ts, facts, program, assertedIndex = null) {
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
  if (
    !signatures.length ||
    (signatures.length > 1 &&
      (assertedIndex === null ||
        !signatures.every((signature) => {
          const predicate = checker.getTypePredicateOfSignature(signature);
          return (
            predicate?.kind === ts.TypePredicateKind.AssertsIdentifier &&
            predicate.parameterIndex === assertedIndex
          );
        })))
  )
    return null;
  const signature = checker.getResolvedSignature(call);
  const declaration = signature?.getDeclaration();
  if (!declaration || !signatures.some((entry) => entry.getDeclaration() === declaration))
    return null;
  const target = stableTarget(
    checker.getSymbolAtLocation(callee),
    callerWrites,
    checker,
    ts,
    facts,
    program,
  );
  if (!target) return null;
  const variable = variableOwner(declaration, ts);
  const matches =
    (target.declarations.includes(declaration) && ts.isFunctionDeclaration(declaration)) ||
    (target.declarations.length === 1 &&
      variable === target.declarations[0] &&
      callableVariable(variable, ts));
  if (!matches) return null;
  const implementation = assertionDeclaration(signature, checker, ts);
  if (
    implementation.asteriskToken ||
    implementation.modifiers?.some((modifier) => modifier.kind === ts.SyntaxKind.AsyncKeyword)
  )
    return null;
  return signature;
}

module.exports = { stableCallSignature, assertionDeclaration };
