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

function aliasTarget(symbol, checker, ts) {
  return symbol.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol;
}

function resolvedSymbol(node, checker, ts) {
  const symbol = ts.isShorthandPropertyAssignment(node.parent)
    ? checker.getShorthandAssignmentValueSymbol(node.parent)
    : checker.getSymbolAtLocation(node);
  return symbol && aliasTarget(symbol, checker, ts);
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

// An append-only, interpolation-free module-level `const` sql tag, imported or not.
function fragmentName(node, ts) {
  if (ts.isPropertyAccessExpression(node)) return node.name;
  if (
    ts.isElementAccessExpression(node) &&
    node.argumentExpression &&
    ts.isStringLiteralLike(node.argumentExpression)
  )
    return node.argumentExpression;
  return node;
}

function tsConstFragment(node, shared, judge) {
  const { checker, ts } = shared;
  const symbol = resolvedSymbol(fragmentName(node, ts), checker, ts);
  const declaration = symbol?.declarations?.length === 1 ? symbol.declarations[0] : null;
  return Boolean(declaration && tsConstDeclaration(declaration, ts, judge));
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

function exportClause(variable) {
  return variable.references.every(
    (reference) =>
      reference.init ||
      appendArgument(reference.identifier) ||
      reference.identifier.parent.type === "ExportSpecifier",
  );
}

// An appended fragment: a never-written module-level const sql tag in this file.
// An imported interpolation-free const is accepted without reading other files.
function stringMember(node) {
  return (
    node.type === "MemberExpression" &&
    node.computed &&
    !node.optional &&
    node.object.type === "Identifier" &&
    node.property.type === "Literal" &&
    typeof node.property.value === "string"
  );
}

function sharedServices(context) {
  const services = context.sourceCode.parserServices;
  if (!services?.program) return null;
  return {
    services,
    shared: {
      checker: services.program.getTypeChecker(),
      ts: require("typescript"),
      program: services.program,
    },
  };
}

function programConst(variable) {
  const [def] = variable.defs;
  return Boolean(
    def?.type === "Variable" &&
    def.parent.kind === "const" &&
    (def.parent.parent.type === "Program" ||
      (def.parent.parent.type === "ExportNamedDeclaration" &&
        def.parent.parent.parent?.type === "Program")),
  );
}

function constFragment(node, context, judges) {
  const variable = node.type === "Identifier" ? resolveVariable(node, context) : null;
  if (variable && localConst(variable, judges.local)) return true;
  if (
    variable &&
    programConst(variable) &&
    !(typedReady(context, node) && exportClause(variable))
  ) {
    return false;
  }
  const reference =
    node.type === "Identifier" ||
    stringMember(node) ||
    (node.type === "MemberExpression" && !node.computed && node.object.type === "Identifier");
  if (!reference || !typedReady(context, node)) return false;
  const typed = sharedServices(context);
  return tsConstFragment(
    typed.services.esTreeNodeToTSNodeMap.get(node),
    typed.shared,
    judges.ts(typed.shared),
  );
}

function typedReady(context, node) {
  const typed = sharedServices(context);
  return Boolean(typed && typed.services.esTreeNodeToTSNodeMap?.has(node));
}

module.exports = { constFragment, isAppend, tsConstFragment };
