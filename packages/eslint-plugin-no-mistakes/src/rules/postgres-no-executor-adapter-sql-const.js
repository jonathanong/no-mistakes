"use strict";

const { resolveVariable } = require("./postgres-query-text");

// `statement.append(fragment)` stays static when both sides are static SQL; see
// "Append chains" in docs/eslint-rules/postgres-no-executor-adapter.md.
function isAppend(node) {
  return (
    node?.type === "CallExpression" &&
    node.arguments.length === 1 &&
    node.callee.type === "MemberExpression" &&
    !node.callee.computed &&
    node.callee.property.name === "append"
  );
}

function appendArgument(identifier) {
  return isAppend(identifier.parent) && identifier.parent.arguments[0] === identifier;
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

function tsConstDeclaration(declaration, ts, judge) {
  const list = declaration.parent;
  return Boolean(
    ts.isVariableDeclaration(declaration) &&
    ts.isIdentifier(declaration.name) &&
    list.flags & ts.NodeFlags.Const &&
    ts.isVariableStatement(list.parent) &&
    ts.isSourceFile(list.parent.parent) &&
    declaration.initializer &&
    ts.isTaggedTemplateExpression(declaration.initializer) &&
    judge(declaration.initializer),
  );
}

function resolvedSymbol(node, checker, ts) {
  const symbol = ts.isShorthandPropertyAssignment(node.parent)
    ? checker.getShorthandAssignmentValueSymbol(node.parent)
    : checker.getSymbolAtLocation(node);
  return symbol && symbol.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol;
}

function aliasTarget(symbol, checker, ts) {
  return symbol.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol;
}

// A namespace import used as anything but `ns.X` hands every export to code
// this pass cannot follow (destructuring, `ns["X"]`, passing, spreading).
function escapeNamespace(node, escaped, checker, ts) {
  const declaration = node.parent;
  if (ts.isNamespaceImport(declaration) || ts.isNamespaceExport(declaration)) return;
  const module = resolvedSymbol(node, checker, ts);
  if (!module || !(module.flags & ts.SymbolFlags.Module)) return;
  if (ts.isPropertyAccessExpression(declaration) && declaration.expression === node) return;
  for (const member of checker.getExportsOfModule(module)) {
    escaped.add(member);
    escaped.add(aliasTarget(member, checker, ts));
  }
}

// `const { X } = value` reads property X of any value, so X escapes.
function escapeDestructured(node, escaped, checker, ts) {
  const element = node.parent;
  if (!ts.isBindingElement(element) || element.name !== node) return;
  const key = element.propertyName ?? node;
  if (!ts.isIdentifier(key) && !ts.isStringLiteral(key)) return;
  const property = checker.getTypeAtLocation(element.parent).getProperty(key.text);
  if (property) escaped.add(aliasTarget(property, checker, ts));
}

// Symbols with a use that is not an `.append` argument, resolved through every
// import, re-export and rename alias, so no text search can hide a mutation.
function escapedSymbols(program, checker, ts) {
  const escaped = new Set();
  for (const file of program.getSourceFiles()) {
    if (program.isSourceFileFromExternalLibrary(file) || program.isSourceFileDefaultLibrary(file))
      continue;
    (function visit(node) {
      if (
        ts.isIdentifier(node) &&
        !(ts.isVariableDeclaration(node.parent) && node.parent.name === node)
      ) {
        const reference = ts.isPropertyAccessExpression(node.parent) && node.parent.name === node;
        const use = reference ? node.parent : node;
        const specifier = ts.isImportSpecifier(node.parent) || ts.isExportSpecifier(node.parent);
        escapeNamespace(node, escaped, checker, ts);
        escapeDestructured(node, escaped, checker, ts);
        const symbol = resolvedSymbol(node, checker, ts);
        if (symbol && !specifier && !tsAppendArgument(use, ts)) escaped.add(symbol);
      }
      ts.forEachChild(node, visit);
    })(file);
  }
  return escaped;
}

const escapedByProgram = new WeakMap();

// Every program-wide use of the const is the argument of an `.append` call.
function appendOnly(declaration, checker, ts, program) {
  if (!escapedByProgram.has(program))
    escapedByProgram.set(program, escapedSymbols(program, checker, ts));
  return !escapedByProgram.get(program).has(checker.getSymbolAtLocation(declaration.name));
}

// An appendOnly, interpolation-free module-level `const` sql tag, imported or not.
function tsConstFragment(node, shared, judge) {
  const { checker, ts, program } = shared;
  const name = ts.isPropertyAccessExpression(node) ? node.name : node;
  const symbol = resolvedSymbol(name, checker, ts);
  const declaration = symbol?.declarations?.length === 1 ? symbol.declarations[0] : null;
  return Boolean(
    declaration &&
    tsConstDeclaration(declaration, ts, judge) &&
    appendOnly(declaration, checker, ts, program),
  );
}

function localConst(variable, judge) {
  const [def] = variable.defs;
  return Boolean(
    def?.type === "Variable" &&
    def.parent.kind === "const" &&
    def.parent.parent.type === "Program" &&
    def.node.id.type === "Identifier" &&
    def.node.init?.type === "TaggedTemplateExpression" &&
    judge(def.node.init) &&
    variable.references.every(
      (reference) => reference.init || appendArgument(reference.identifier),
    ),
  );
}

// An appended fragment: a never-written, never-mutated module-level const sql tag.
// Constants this file cannot see all uses of go through the type checker.
function constFragment(node, context, judges) {
  const variable = node.type === "Identifier" ? resolveVariable(node, context) : null;
  if (variable && localConst(variable, judges.local)) return true;
  const services = context.sourceCode.parserServices;
  const reference =
    node.type === "Identifier" ||
    (node.type === "MemberExpression" && !node.computed && node.object.type === "Identifier");
  if (!reference || !services?.program || !services.esTreeNodeToTSNodeMap?.has(node)) return false;
  const shared = {
    checker: services.program.getTypeChecker(),
    ts: require("typescript"),
    program: services.program,
  };
  return tsConstFragment(services.esTreeNodeToTSNodeMap.get(node), shared, judges.ts(shared));
}

module.exports = { constFragment, isAppend, tsConstFragment };
