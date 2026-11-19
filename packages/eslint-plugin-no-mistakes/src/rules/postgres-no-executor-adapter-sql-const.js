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

function constFragment(node, context, judges) {
  const variable = node.type === "Identifier" ? resolveVariable(node, context) : null;
  const typed = sharedServices(context);
  if (variable && localConst(variable, judges.local)) {
    if (!typed || !typed.services.esTreeNodeToTSNodeMap?.has(node)) return true;
    const symbol = resolvedSymbol(
      typed.services.esTreeNodeToTSNodeMap.get(node),
      typed.shared.checker,
      typed.shared.ts,
    );
    return !symbol || !escapes(symbol, typed.shared);
  }
  const reference =
    node.type === "Identifier" ||
    stringMember(node) ||
    (node.type === "MemberExpression" && !node.computed && node.object.type === "Identifier");
  if (!reference || !typed || !typed.services.esTreeNodeToTSNodeMap?.has(node)) return false;
  return tsConstFragment(
    typed.services.esTreeNodeToTSNodeMap.get(node),
    typed.shared,
    judges.ts(typed.shared),
  );
}

module.exports = { constFragment, isAppend, tsConstFragment };
