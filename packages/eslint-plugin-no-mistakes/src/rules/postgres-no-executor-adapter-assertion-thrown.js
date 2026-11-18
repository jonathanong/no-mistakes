"use strict";

const { primitiveType } = require("./postgres-no-executor-adapter-sql-primitive");

function literal(node, ts) {
  return (
    ts.isStringLiteral(node) ||
    ts.isNumericLiteral(node) ||
    node.kind === ts.SyntaxKind.TrueKeyword ||
    node.kind === ts.SyntaxKind.FalseKeyword ||
    node.kind === ts.SyntaxKind.NullKeyword
  );
}

function assertionThrownValue(node, checker, ts, program, writes, parameters, intrinsics) {
  function intrinsic(expression) {
    const symbol = checker.getSymbolAtLocation(expression);
    intrinsics.add(symbol);
    return Boolean(
      symbol?.declarations?.length &&
      !writes.symbols.has(symbol) &&
      !writes.escapedSymbols.has(symbol) &&
      symbol.declarations.every(
        (entry) =>
          program.isSourceFileDefaultLibrary(entry.getSourceFile()) &&
          !writes.declarations.has(entry),
      ),
    );
  }

  function freshError(expression) {
    return (
      ts.isNewExpression(expression) &&
      ts.isIdentifier(expression.expression) &&
      expression.expression.text === "Error" &&
      expression.arguments?.every((argument) => literal(argument, ts)) &&
      intrinsic(expression.expression)
    );
  }

  function scalar(value) {
    return (
      literal(value, ts) ||
      (ts.isIdentifier(value) &&
        parameters.has(
          checker.getShorthandAssignmentValueSymbol(value.parent) ??
            checker.getSymbolAtLocation(value),
        ) &&
        primitiveType(checker.getTypeAtLocation(value), ts, true))
    );
  }

  function metadata(expression) {
    return (
      ts.isObjectLiteralExpression(expression) &&
      expression.properties.every((property) => {
        if (ts.isShorthandPropertyAssignment(property))
          return !property.objectAssignmentInitializer && scalar(property.name);
        return (
          ts.isPropertyAssignment(property) &&
          (ts.isIdentifier(property.name) ||
            ts.isStringLiteral(property.name) ||
            ts.isNumericLiteral(property.name)) &&
          scalar(property.initializer)
        );
      })
    );
  }

  if (literal(node, ts) || freshError(node)) return true;
  return (
    ts.isCallExpression(node) &&
    !node.questionDotToken &&
    ts.isPropertyAccessExpression(node.expression) &&
    !node.expression.questionDotToken &&
    ts.isIdentifier(node.expression.expression) &&
    node.expression.expression.text === "Object" &&
    node.expression.name.text === "assign" &&
    node.arguments.length === 2 &&
    intrinsic(node.expression.expression) &&
    intrinsic(node.expression) &&
    freshError(node.arguments[0]) &&
    metadata(node.arguments[1])
  );
}

module.exports = { assertionThrownValue, literal };
