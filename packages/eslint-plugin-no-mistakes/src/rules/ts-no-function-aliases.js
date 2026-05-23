"use strict";

const { rule } = require("../helpers");

function unwrapExpression(expression) {
  let current = expression;
  while (
    current &&
    (current.type === "AwaitExpression" ||
      current.type === "ChainExpression" ||
      current.type === "TSNonNullExpression")
  ) {
    current = current.type === "AwaitExpression" ? current.argument : current.expression;
  }
  return current;
}

function expressionStatementCall(statement) {
  if (statement.type !== "ExpressionStatement") return null;
  return unwrapExpression(statement.expression);
}

function returnStatementCall(statement) {
  if (statement.type !== "ReturnStatement") return null;
  return unwrapExpression(statement.argument);
}

function onlyCallExpression(body) {
  if (!body) return null;
  if (body.type !== "BlockStatement") {
    return unwrapExpression(body);
  }
  if (body.body.length !== 1) return null;
  const statement = body.body[0];
  return returnStatementCall(statement) || expressionStatementCall(statement);
}

function parameterName(param) {
  const unwrapped = param.type === "AssignmentPattern" ? param.left : param;
  return unwrapped.type === "Identifier" ? unwrapped.name : null;
}

function isSameArgumentList(params, args) {
  if (params.length !== args.length) return false;
  return params.every((param, index) => {
    const name = parameterName(param);
    return name && args[index].type === "Identifier" && args[index].name === name;
  });
}

function isSelfCall(node, call) {
  if (call.callee.type !== "Identifier") return false;
  if (node.id && node.id.name === call.callee.name) return true;
  const parent = node.parent;
  return (
    parent &&
    parent.type === "VariableDeclarator" &&
    parent.id.type === "Identifier" &&
    parent.id.name === call.callee.name
  );
}

function reportIfAlias(node, context) {
  const call = onlyCallExpression(node.body);
  if (!call || call.type !== "CallExpression") return;
  if (isSelfCall(node, call)) return;
  if (isSameArgumentList(node.params || [], call.arguments)) {
    context.report({ node, messageId: "alias" });
  }
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow function wrappers that only alias another function",
      recommended: true,
    },
    schema: [],
    messages: {
      alias:
        "Do not create a function that only aliases another function call. Export or call the original function name directly so agents can trace behavior.",
    },
  },
  (context) => ({
    ArrowFunctionExpression(node) {
      reportIfAlias(node, context);
    },
    FunctionDeclaration(node) {
      reportIfAlias(node, context);
    },
    FunctionExpression(node) {
      reportIfAlias(node, context);
    },
  }),
);
