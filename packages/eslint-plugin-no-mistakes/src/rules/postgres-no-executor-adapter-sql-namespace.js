"use strict";

// Module object a binding refers to. `import *`, `import = require`, a dynamic
// `import()`, and `require()` of a string specifier. See the append-chain
// section of docs/eslint-rules/postgres-no-executor-adapter.md.
const resolvedModules = new WeakMap();

function aliasTarget(symbol, checker, ts) {
  return symbol.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol;
}

function moduleSymbol(symbol, checker, ts) {
  const resolved = symbol && aliasTarget(symbol, checker, ts);
  return resolved?.flags & ts.SymbolFlags.Module ? resolved : null;
}

function stringSpecifier(node, ts) {
  if (!node) return null;
  if (ts.isAwaitExpression(node) || ts.isParenthesizedExpression(node))
    return stringSpecifier(node.expression, ts);
  if (!ts.isCallExpression(node) || node.arguments.length === 0) return null;
  const spec = node.arguments[0];
  if (!ts.isStringLiteralLike(spec)) return null;
  if (node.expression.kind === ts.SyntaxKind.ImportKeyword) return spec.text;
  if (ts.isIdentifier(node.expression) && node.expression.text === "require") return spec.text;
  return null;
}

function moduleFromSpecifier(spec, file, program, checker, ts) {
  let cache = resolvedModules.get(program);
  if (!cache) resolvedModules.set(program, (cache = new Map()));
  const key = `${file.fileName}\0${spec}`;
  if (cache.has(key)) return cache.get(key);
  const host = ts.createCompilerHost(program.getCompilerOptions());
  const resolved = ts.resolveModuleName(spec, file.fileName, program.getCompilerOptions(), host);
  const name = resolved.resolvedModule?.resolvedFileName;
  const source = name && program.getSourceFile(name);
  const module = source ? moduleSymbol(checker.getSymbolAtLocation(source), checker, ts) : null;
  cache.set(key, module);
  return module;
}

function typeModule(node, checker, ts) {
  return moduleSymbol(checker.getTypeAtLocation(node).symbol, checker, ts);
}

function declaredModule(node, shared) {
  const { checker, ts, program } = shared;
  const symbol = checker.getSymbolAtLocation(node);
  const direct = moduleSymbol(symbol, checker, ts);
  if (direct) return direct;
  const declaration = symbol?.valueDeclaration;
  if (declaration && ts.isVariableDeclaration(declaration) && program) {
    const spec = stringSpecifier(declaration.initializer, ts);
    if (spec) return moduleFromSpecifier(spec, declaration.getSourceFile(), program, checker, ts);
    return typeModule(node, checker, ts);
  }
  if (declaration && ts.isParameter(declaration)) return typeModule(node, checker, ts);
  return null;
}

function memberText(node, ts) {
  const parent = node.parent;
  if (ts.isPropertyAccessExpression(parent) && parent.expression === node) return parent.name.text;
  if (
    ts.isElementAccessExpression(parent) &&
    parent.expression === node &&
    parent.argumentExpression &&
    ts.isStringLiteralLike(parent.argumentExpression)
  )
    return parent.argumentExpression.text;
  return null;
}

module.exports = { declaredModule, memberText };
