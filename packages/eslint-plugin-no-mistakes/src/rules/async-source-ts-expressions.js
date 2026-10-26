"use strict";

function expression(node, api) {
  const { ts, convert: c, make: m, list, fn, property } = api;
  const kind = ts.SyntaxKind;
  if (ts.isIdentifier(node)) return m(node, "Identifier", { name: node.text });
  if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node))
    return m(node, "Literal", { value: node.text });
  if (ts.isNumericLiteral(node)) return m(node, "Literal", { value: Number(node.text) });
  if ([kind.TrueKeyword, kind.FalseKeyword, kind.NullKeyword].includes(node.kind))
    return m(node, "Literal", {
      value: node.kind === kind.NullKeyword ? null : node.kind === kind.TrueKeyword,
    });
  if (node.kind === kind.ThisKeyword) return m(node, "ThisExpression");
  if (ts.isArrowFunction(node)) return fn(node, "ArrowFunctionExpression");
  if (ts.isFunctionExpression(node)) return fn(node, "FunctionExpression");
  let result;
  if (ts.isCallExpression(node) || ts.isNewExpression(node)) {
    result = m(node, ts.isNewExpression(node) ? "NewExpression" : "CallExpression", {
      callee: c(node.expression),
      arguments: list(node.arguments),
      optional: Boolean(node.questionDotToken),
      typeArguments: typeArguments(node, api),
    });
  } else if (ts.isPropertyAccessExpression(node) || ts.isElementAccessExpression(node)) {
    result = m(node, "MemberExpression", {
      object: c(node.expression),
      property: c(node.name || node.argumentExpression),
      computed: ts.isElementAccessExpression(node),
      optional: Boolean(node.questionDotToken),
    });
  }
  if (result) {
    if (ts.isOptionalChain(node) && !ts.isOptionalChain(node.parent))
      return m(node, "ChainExpression", { expression: result });
    return result;
  }
  if (ts.isAwaitExpression(node) || ts.isYieldExpression(node))
    return m(node, ts.isAwaitExpression(node) ? "AwaitExpression" : "YieldExpression", {
      argument: c(node.expression),
      delegate: Boolean(node.asteriskToken),
    });
  if (
    ts.isAsExpression(node) ||
    ts.isTypeAssertionExpression(node) ||
    ts.isSatisfiesExpression?.(node)
  )
    return m(
      node,
      ts.isAsExpression(node)
        ? "TSAsExpression"
        : ts.isTypeAssertionExpression(node)
          ? "TSTypeAssertion"
          : "TSSatisfiesExpression",
      {
        expression: c(node.expression),
        typeAnnotation: c(node.type),
      },
    );
  if (ts.isNonNullExpression(node))
    return m(node, "TSNonNullExpression", { expression: c(node.expression) });
  if (ts.isSpreadElement(node) || ts.isSpreadAssignment(node))
    return m(node, "SpreadElement", { argument: c(node.expression) });
  if (ts.isArrayLiteralExpression(node))
    return m(node, "ArrayExpression", {
      elements: Array.from(node.elements, (item) =>
        ts.isOmittedExpression(item) ? null : c(item),
      ),
    });
  if (ts.isObjectLiteralExpression(node))
    return m(node, "ObjectExpression", { properties: list(node.properties) });
  if (ts.isPropertyAssignment(node) || ts.isShorthandPropertyAssignment(node)) {
    if (node.objectAssignmentInitializer) throw new Error("Unsupported shorthand assignment");
    return m(node, "Property", {
      ...property(node),
      value: c(node.initializer || node.name),
      shorthand: ts.isShorthandPropertyAssignment(node),
      method: false,
      kind: "init",
    });
  }
  if (ts.isMethodDeclaration(node) || ts.isGetAccessor(node) || ts.isSetAccessor(node))
    return m(node, "Property", {
      ...property(node),
      value: fn(node, "FunctionExpression"),
      method: ts.isMethodDeclaration(node),
      shorthand: false,
      kind: ts.isGetAccessor(node) ? "get" : ts.isSetAccessor(node) ? "set" : "init",
    });
  if (ts.isConditionalExpression(node))
    return m(node, "ConditionalExpression", {
      test: c(node.condition),
      consequent: c(node.whenTrue),
      alternate: c(node.whenFalse),
    });
  if (ts.isBinaryExpression(node)) {
    const token = node.operatorToken.kind;
    const operator = ts.tokenToString(token);
    if (token === kind.CommaToken)
      return m(node, "SequenceExpression", { expressions: [c(node.left), c(node.right)] });
    const assignment = token >= kind.FirstAssignment && token <= kind.LastAssignment;
    return m(
      node,
      assignment
        ? "AssignmentExpression"
        : [kind.AmpersandAmpersandToken, kind.BarBarToken, kind.QuestionQuestionToken].includes(
              token,
            )
          ? "LogicalExpression"
          : "BinaryExpression",
      {
        operator,
        left: assignment ? assignmentTarget(node.left, api) : c(node.left),
        right: c(node.right),
      },
    );
  }
  if (ts.isPrefixUnaryExpression(node) || ts.isPostfixUnaryExpression(node)) {
    const operator = ts.tokenToString(node.operator);
    return m(node, ["++", "--"].includes(operator) ? "UpdateExpression" : "UnaryExpression", {
      operator,
      prefix: ts.isPrefixUnaryExpression(node),
      argument: c(node.operand),
    });
  }
  const unary = new Map([
    [kind.VoidExpression, "void"],
    [kind.DeleteExpression, "delete"],
    [kind.TypeOfExpression, "typeof"],
  ]);
  if (unary.has(node.kind))
    return m(node, "UnaryExpression", {
      operator: unary.get(node.kind),
      prefix: true,
      argument: c(node.expression),
    });
  return null;
}

function typeArguments(node, { make, list }) {
  return (
    node.typeArguments &&
    make(node, "TSTypeParameterInstantiation", { params: list(node.typeArguments) })
  );
}

function assignmentTarget(node, api) {
  const { ts, convert: c, make: m } = api;
  if (ts.isParenthesizedExpression(node)) return assignmentTarget(node.expression, api);
  if (ts.isArrayLiteralExpression(node))
    return m(node, "ArrayPattern", {
      elements: Array.from(node.elements, (item) =>
        ts.isOmittedExpression(item) ? null : assignmentTarget(item, api),
      ),
    });
  if (ts.isObjectLiteralExpression(node))
    return m(node, "ObjectPattern", {
      properties: Array.from(node.properties, (item) => {
        if (ts.isSpreadAssignment(item)) return assignmentTarget(item, api);
        if (!ts.isPropertyAssignment(item) && !ts.isShorthandPropertyAssignment(item))
          throw new Error("Unsupported assignment property");
        let value = assignmentTarget(item.initializer || item.name, api);
        if (item.objectAssignmentInitializer)
          value = m(item, "AssignmentPattern", {
            left: value,
            right: c(item.objectAssignmentInitializer),
          });
        return m(item, "Property", {
          ...api.property(item),
          value,
          shorthand: ts.isShorthandPropertyAssignment(item),
          method: false,
          kind: "init",
        });
      }),
    });
  if (ts.isSpreadElement(node) || ts.isSpreadAssignment(node))
    return m(node, "RestElement", { argument: assignmentTarget(node.expression, api) });
  if (ts.isBinaryExpression(node) && node.operatorToken.kind === ts.SyntaxKind.EqualsToken)
    return m(node, "AssignmentPattern", {
      left: assignmentTarget(node.left, api),
      right: c(node.right),
    });
  return c(node);
}

module.exports = { expression, typeArguments, assignmentTarget };
