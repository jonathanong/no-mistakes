"use strict";

const { tsLiteralSql } = require("./postgres-no-executor-adapter-sql");
const { tsHoleSafe } = require("./postgres-no-executor-adapter-sql-origin");
const { memberWrite } = require("./postgres-no-executor-adapter-option-forward");
const { sourceWrites } = require("./postgres-no-executor-adapter-source");

// An argument that is not a plain, unwritten parameter of the forwarding
// function has no scalar proof, so the hole check rejects it.
const UNPROVEN = { type: "ThisExpression" };

function forwardedArguments(invoked, declaration, outer, shared) {
  const { checker, ts, facts, program } = shared;
  const writes = sourceWrites(declaration.getSourceFile(), checker, facts, program).symbols;
  return invoked.arguments.map((argument) => {
    const symbol = ts.isIdentifier(argument) && checker.getSymbolAtLocation(argument);
    const index = declaration.parameters.findIndex(
      (param) => ts.isIdentifier(param.name) && checker.getSymbolAtLocation(param.name) === symbol,
    );
    const param = declaration.parameters[index];
    return symbol && !writes.has(symbol) && !param?.initializer && outer[index]
      ? outer[index]
      : UNPROVEN;
  });
}

function parameterBinding(parameter, property, ts) {
  if (property && ts.isObjectBindingPattern(parameter.name)) {
    const element = parameter.name.elements.find(
      (entry) =>
        ts.isIdentifier(entry.name) &&
        (entry.propertyName?.text ?? entry.name.text) === property &&
        !entry.initializer &&
        !entry.dotDotDotToken,
    );
    return { binding: element?.name, object: false };
  }
  return { binding: parameter.name, object: Boolean(property) };
}

// `args` are the ESTree arguments the executor's SQL holes are judged against:
// the original call's, or for a pass-on the arguments forwarded by it.
function declarationVisible(declaration, index, property, args, shared) {
  const { context, options, facts, checker, ts, program } = shared;
  const parameter = declaration?.parameters?.[index];
  if (!declaration?.body || !parameter) return null;
  const { binding, object } = parameterBinding(parameter, property, ts);
  if (!binding || !ts.isIdentifier(binding)) return null;
  const symbol = checker.getSymbolAtLocation(binding);
  if (!symbol) return null;
  let seen = false;
  let safe = true;
  function passedOn(node, invoked) {
    const inner = checker.getResolvedSignature(invoked)?.getDeclaration();
    const forwarded = forwardedArguments(invoked, declaration, args, shared);
    const at = invoked.arguments.indexOf(node);
    return declarationVisible(inner, at, property, forwarded, shared) === true;
  }
  function visit(node) {
    if (!safe) return;
    if (ts.isIdentifier(node) && node.text === "arguments") safe = false;
    if (node !== binding && ts.isIdentifier(node) && checker.getSymbolAtLocation(node) === symbol) {
      seen = true;
      let invoked = node.parent;
      if (
        object &&
        ts.isPropertyAccessExpression(invoked) &&
        invoked.expression === node &&
        invoked.name.text !== property
      ) {
        if (memberWrite(invoked)) safe = false;
        return;
      }
      let callee = node;
      if (
        object &&
        ts.isPropertyAccessExpression(invoked) &&
        invoked.expression === node &&
        invoked.name.text === property
      ) {
        callee = invoked;
        invoked = invoked.parent;
      }
      while (ts.isNonNullExpression(invoked)) {
        callee = invoked;
        invoked = invoked.parent;
      }
      if (object && ts.isCallExpression(invoked) && invoked.arguments.includes(node)) {
        if (!passedOn(node, invoked)) safe = false;
        return;
      }
      const hole = (expression) =>
        tsHoleSafe(
          expression,
          { arguments: args },
          declaration,
          checker,
          ts,
          context,
          facts,
          program,
          symbol,
          property,
        );
      if (
        !ts.isCallExpression(invoked) ||
        invoked.expression !== callee ||
        !invoked.arguments[0] ||
        !tsLiteralSql(invoked.arguments[0], checker, ts, options, hole)
      ) {
        safe = false;
      }
    }
    ts.forEachChild(node, visit);
  }
  visit(declaration.body);
  return seen && safe;
}

function importedVisible(call, index, property, context, options, facts) {
  const services = context.sourceCode.parserServices;
  if (!services?.program || !services.esTreeNodeToTSNodeMap) return null;
  const checker = services.program.getTypeChecker();
  const callee = services.esTreeNodeToTSNodeMap.get(call.callee);
  const signatures = checker.getTypeAtLocation(callee).getCallSignatures();
  if (signatures.length !== 1) return null;
  const shared = { context, options, facts, checker, program: services.program };
  shared.ts = require("typescript");
  const declaration = signatures[0].getDeclaration();
  return declarationVisible(declaration, index, property, call.arguments, shared);
}

module.exports = { importedVisible };
