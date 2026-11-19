"use strict";

const { indexFor, programFiles } = require("./postgres-no-executor-adapter-sql-index");

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

// Namespace aliases of modules exporting the candidate, plus their re-exports.
function namespaceNames(index, candidate, names, shared) {
  const { checker, ts } = shared;
  const result = new Set();
  for (const name of index.namespaces) {
    if (!index.modules.has(name)) index.modules.set(name, resolvedSymbol(name, checker, ts));
    const module = index.modules.get(name);
    if (!(module?.flags & ts.SymbolFlags.Module) || !moduleHas(module, candidate, names, shared))
      continue;
    for (const alias of aliasNames(index, [name.text], module, shared)) result.add(alias);
  }
  return result;
}

function escapesNamespace(node, candidate, names, shared) {
  const { checker, ts } = shared;
  const declaration = node.parent;
  if (ts.isNamespaceImport(declaration) || ts.isNamespaceExport(declaration)) return false;
  const module = resolvedSymbol(node, checker, ts);
  if (!(module?.flags & ts.SymbolFlags.Module)) return false;
  if (ts.isPropertyAccessExpression(declaration) && declaration.expression === node) return false;
  return moduleHas(module, candidate, names, shared);
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

function escapesUse(node, state) {
  const { candidate, names, spaces, shared } = state;
  const { checker, ts } = shared;
  if (ts.isVariableDeclaration(node.parent) && node.parent.name === node) return false;
  if (spaces.has(node.text) && escapesNamespace(node, candidate, names, shared)) return true;
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
  const spaces = namespaceNames(index, candidate, names, shared);
  const texts = [...names, ...spaces];
  const walk = walker({ candidate, names, spaces, shared });
  return programFiles(shared.program).some(
    (file) => texts.some((text) => file.text.includes(text)) && walk(file),
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
