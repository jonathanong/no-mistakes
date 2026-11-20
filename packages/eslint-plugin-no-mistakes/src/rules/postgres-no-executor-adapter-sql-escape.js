"use strict";

const { indexFor, programFiles } = require("./postgres-no-executor-adapter-sql-index");
const { declaredModule, memberText } = require("./postgres-no-executor-adapter-sql-namespace");

const verdicts = new WeakMap();

function aliasTarget(symbol, checker, ts) {
  return symbol.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol;
}

function resolvedSymbol(node, checker, ts) {
  const symbol = ts.isShorthandPropertyAssignment(node.parent)
    ? checker.getShorthandAssignmentValueSymbol(node.parent)
    : checker.getSymbolAtLocation(node);
  return symbol && aliasTarget(symbol, checker, ts);
}

function tsAppendArgument(use, ts) {
  const call = use.parent;
  return Boolean(
    ts.isCallExpression(call) &&
    call.arguments.length === 1 &&
    call.arguments[0] === use &&
    ts.isPropertyAccessExpression(call.expression) &&
    call.expression.name.text === "append",
  );
}

// Names a symbol is reachable under: its own and every import/export
// specifier that resolves back to it. Same-text specifiers of other symbols
// fail the checker comparison, so only real aliases are followed.
function aliasNames(index, seeds, target, shared) {
  const { checker, ts } = shared;
  const names = new Set(seeds);
  const pending = [...seeds];
  while (pending.length) {
    for (const local of index.bySource.get(pending.pop()) ?? []) {
      if (names.has(local.text) || resolvedSymbol(local, checker, ts) !== target) continue;
      names.add(local.text);
      pending.push(local.text);
    }
  }
  return names;
}

function moduleHas(module, candidate, names, shared) {
  const { checker, ts } = shared;
  for (const name of names) {
    const member = checker.tryGetMemberInModuleExports(name, module);
    if (member && aliasTarget(member, checker, ts) === candidate) return true;
  }
  return false;
}

function filterTexts(index, names) {
  const texts = new Set(names);
  const pending = [];
  for (const name of index.namespaces) {
    if (!texts.has(name.text)) pending.push(name.text);
    texts.add(name.text);
  }
  while (pending.length) {
    for (const alias of index.bySource.get(pending.pop()) ?? []) {
      if (texts.has(alias.text)) continue;
      texts.add(alias.text);
      pending.push(alias.text);
    }
  }
  return [...texts];
}

function namespaceEscape(node, candidate, names, shared) {
  const { ts } = shared;
  const parent = node.parent;
  if (ts.isNamespaceImport(parent) || ts.isNamespaceExport(parent)) return false;
  if (ts.isImportEqualsDeclaration(parent) && parent.name === node) return false;
  const module = declaredModule(node, shared);
  if (!module) return false;
  const member = memberText(node, ts);
  if (member === null) return moduleHas(module, candidate, names, shared);
  if (!names.has(member)) return false;
  const access = parent;
  return moduleHas(module, candidate, new Set([member]), shared) && !tsAppendArgument(access, ts);
}

function keyText(element, ts) {
  const key = element.propertyName ?? element.name;
  return ts.isIdentifier(key) || ts.isStringLiteral(key) ? key.text : undefined;
}

function escapesDestructured(element, candidate, shared) {
  const { checker, ts } = shared;
  const property = checker.getTypeAtLocation(element.parent).getProperty(keyText(element, ts));
  return Boolean(property && aliasTarget(property, checker, ts) === candidate);
}

function inTypePosition(node, ts) {
  for (let current = node.parent; current; current = current.parent) {
    if (current.kind === ts.SyntaxKind.TypeQuery) return true;
    // TS 6 reports class `extends` expressions as type nodes. They still run.
    if (
      current.kind === ts.SyntaxKind.ExpressionWithTypeArguments &&
      ts.isExpressionWithTypeArgumentsInClassExtendsClause(current)
    ) {
      continue;
    }
    if (ts.isTypeNode(current)) return true;
  }
  return false;
}

function escapesUse(node, state) {
  const { candidate, names, shared } = state;
  const { checker, ts } = shared;
  if (inTypePosition(node, ts)) return false;
  if (ts.isVariableDeclaration(node.parent) && node.parent.name === node) return false;
  // The parameter name is the binding. The body uses are the module reads.
  if (ts.isParameter(node.parent) && node.parent.name === node) return false;
  if (namespaceEscape(node, candidate, names, shared)) return true;
  if (!names.has(node.text)) return false;
  const reference = ts.isPropertyAccessExpression(node.parent) && node.parent.name === node;
  const use = reference ? node.parent : node;
  const specifier = ts.isImportSpecifier(node.parent) || ts.isExportSpecifier(node.parent);
  return (
    !specifier && !tsAppendArgument(use, ts) && resolvedSymbol(node, checker, ts) === candidate
  );
}

function walker(state) {
  const { ts } = state.shared;
  return function walk(node) {
    if (ts.isBindingElement(node) && ts.isIdentifier(node.name)) {
      if (
        state.names.has(keyText(node, ts)) &&
        escapesDestructured(node, state.candidate, state.shared)
      )
        return true;
    } else if (ts.isIdentifier(node) && escapesUse(node, state)) return true;
    return ts.forEachChild(node, walk);
  };
}

function computeEscape(candidate, shared) {
  const index = indexFor(shared.program, shared.ts);
  const names = aliasNames(index, [candidate.name], candidate, shared);
  const texts = filterTexts(index, names);
  const walk = walker({ candidate, names, shared });
  return programFiles(shared.program).some((file) => fileMentions(file, texts) && walk(file));
}

function fileMentions(file, texts) {
  const text = file.text;
  return (
    text.includes("import(") ||
    text.includes("require(") ||
    texts.some((name) => text.includes(name))
  );
}

// True when the symbol has any program-wide use that is not an `.append`
// argument, resolved through renames, re-exports and namespaces. Memoized.
function escapes(candidate, shared) {
  if (!verdicts.has(shared.program)) verdicts.set(shared.program, new Map());
  const known = verdicts.get(shared.program);
  if (!known.has(candidate)) known.set(candidate, computeEscape(candidate, shared));
  return known.get(candidate);
}

module.exports = { escapes, resolvedSymbol, tsAppendArgument };
