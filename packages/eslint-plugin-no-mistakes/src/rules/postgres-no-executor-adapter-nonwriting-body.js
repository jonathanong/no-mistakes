"use strict";

const { primitiveType } = require("./postgres-no-executor-adapter-sql-primitive");

const unaryOperators = new Set(["!", "+", "-", "~"]);
const binaryOperators = new Set([
  "===",
  "!==",
  "==",
  "!=",
  "<",
  "<=",
  ">",
  ">=",
  "+",
  "-",
  "*",
  "/",
  "%",
  "**",
  "&",
  "|",
  "^",
  "<<",
  ">>",
  ">>>",
  "&&",
  "||",
  "??",
]);

function pureReaderBody(declaration, receiver, fields, checker, ts) {
  const parameters = new Set();
  for (const parameter of declaration.parameters) {
    if (
      checker.getSymbolAtLocation(parameter.name) !== receiver &&
      parameter.type &&
      primitiveType(checker.getTypeFromTypeNode(parameter.type), ts, false)
    ) {
      const symbol = checker.getSymbolAtLocation(parameter.name);
      if (symbol) parameters.add(symbol);
    }
  }

  function scalarExpression(node, locals) {
    if (
      ts.isStringLiteral(node) ||
      ts.isNoSubstitutionTemplateLiteral(node) ||
      ts.isNumericLiteral(node) ||
      ts.isBigIntLiteral(node) ||
      node.kind === ts.SyntaxKind.TrueKeyword ||
      node.kind === ts.SyntaxKind.FalseKeyword ||
      node.kind === ts.SyntaxKind.NullKeyword
    )
      return true;
    if (ts.isIdentifier(node)) return locals.has(checker.getSymbolAtLocation(node));
    if (ts.isPropertyAccessExpression(node))
      return (
        !node.questionDotToken &&
        ts.isIdentifier(node.expression) &&
        checker.getSymbolAtLocation(node.expression) === receiver &&
        fields.has(node.name.text)
      );
    if (
      ts.isParenthesizedExpression(node) ||
      ts.isAsExpression(node) ||
      ts.isTypeAssertionExpression(node) ||
      ts.isNonNullExpression(node) ||
      ts.isSatisfiesExpression(node)
    )
      return scalarExpression(node.expression, locals);
    if (ts.isPrefixUnaryExpression(node))
      return (
        unaryOperators.has(ts.tokenToString(node.operator)) &&
        scalarExpression(node.operand, locals)
      );
    if (ts.isVoidExpression(node) || ts.isTypeOfExpression(node))
      return scalarExpression(node.expression, locals);
    if (ts.isBinaryExpression(node))
      return (
        binaryOperators.has(ts.tokenToString(node.operatorToken.kind)) &&
        scalarExpression(node.left, locals) &&
        scalarExpression(node.right, locals)
      );
    if (ts.isConditionalExpression(node))
      return (
        scalarExpression(node.condition, locals) &&
        scalarExpression(node.whenTrue, locals) &&
        scalarExpression(node.whenFalse, locals)
      );
    if (ts.isTemplateExpression(node))
      return node.templateSpans.every((span) => scalarExpression(span.expression, locals));
    return false;
  }

  function addLocal(identifier, locals) {
    const symbol = checker.getSymbolAtLocation(identifier);
    if (!symbol) return false;
    locals.add(symbol);
    return true;
  }

  function bindFields(pattern, initializer, locals) {
    if (!ts.isIdentifier(initializer) || checker.getSymbolAtLocation(initializer) !== receiver)
      return false;
    return pattern.elements.every((entry) => {
      const key = entry.propertyName ?? entry.name;
      return (
        !entry.dotDotDotToken &&
        !entry.initializer &&
        ts.isIdentifier(entry.name) &&
        (ts.isIdentifier(key) || ts.isStringLiteral(key)) &&
        fields.has(key.text) &&
        addLocal(entry.name, locals)
      );
    });
  }

  function constDeclarations(node, locals) {
    const list = node.declarationList;
    if ((list.flags & ts.NodeFlags.BlockScoped) !== ts.NodeFlags.Const || node.modifiers?.length)
      return false;
    return list.declarations.every(({ name, initializer }) => {
      if (!initializer) return false;
      if (ts.isObjectBindingPattern(name)) return bindFields(name, initializer, locals);
      return (
        ts.isIdentifier(name) && scalarExpression(initializer, locals) && addLocal(name, locals)
      );
    });
  }

  function pureStatement(node, locals) {
    if (ts.isBlock(node)) {
      const blockLocals = new Set(locals);
      return node.statements.every((statement) => pureStatement(statement, blockLocals));
    }
    if (ts.isVariableStatement(node)) return constDeclarations(node, locals);
    if (ts.isReturnStatement(node))
      return !node.expression || scalarExpression(node.expression, locals);
    if (ts.isIfStatement(node))
      return (
        scalarExpression(node.expression, locals) &&
        pureStatement(node.thenStatement, new Set(locals)) &&
        (!node.elseStatement || pureStatement(node.elseStatement, new Set(locals)))
      );
    if (ts.isExpressionStatement(node)) return scalarExpression(node.expression, locals);
    return ts.isEmptyStatement(node);
  }

  return pureStatement(declaration.body, parameters);
}

module.exports = { pureReaderBody };
