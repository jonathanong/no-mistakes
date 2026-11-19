"use strict";

const { resolveVariable } = require("./postgres-query-text");
const { escapes, resolvedSymbol } = require("./postgres-no-executor-adapter-sql-escape");

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

// An appendOnly, interpolation-free module-level `const` sql tag, imported or not.
function tsConstFragment(node, shared, judge) {
  const { checker, ts } = shared;
  const name = ts.isPropertyAccessExpression(node) ? node.name : node;
  const symbol = resolvedSymbol(name, checker, ts);
  const declaration = symbol?.declarations?.length === 1 ? symbol.declarations[0] : null;
  return Boolean(
    declaration && tsConstDeclaration(declaration, ts, judge) && !escapes(symbol, shared),
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
