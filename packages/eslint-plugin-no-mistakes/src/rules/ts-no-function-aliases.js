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

function identifierName(node) {
  return node && node.type === "Identifier" ? node.name : null;
}

function directCalleeName(call) {
  return identifierName(unwrapExpression(call.callee));
}

function isSameArgumentList(params, args) {
  if (params.length !== args.length) return false;
  return params.every((param, index) => {
    if (param.type === "RestElement") {
      return (
        args[index].type === "SpreadElement" &&
        identifierName(param.argument) === identifierName(args[index].argument)
      );
    }
    if (param.type === "AssignmentPattern") return false;
    const name = identifierName(param);
    return name && args[index].type === "Identifier" && args[index].name === name;
  });
}

function isSelfCall(node, call) {
  const callee = directCalleeName(call);
  if (!callee) return false;
  const wrapper = variableWrapperName(node);
  if (wrapper) return wrapper === callee;
  if (node.id && node.id.name === callee) return true;
  const parent = node.parent;
  return (
    parent &&
    parent.type === "VariableDeclarator" &&
    parent.id.type === "Identifier" &&
    parent.id.name === callee
  );
}

function assignmentWrapperName(node) {
  if (node.parent?.type !== "AssignmentExpression" || node.parent.right !== node) return null;
  const left = node.parent.left;
  if (left.type === "Identifier") return left.name;
  if (left.type === "MemberExpression" && !left.computed) return identifierName(left.property);
  if (left.type === "MemberExpression" && left.property.type === "Literal") {
    return String(left.property.value);
  }
  return null;
}

function variableWrapperName(node) {
  return node.parent?.type === "VariableDeclarator" && node.parent.id.type === "Identifier"
    ? node.parent.id.name
    : null;
}

function wrapperName(node) {
  return variableWrapperName(node) || assignmentWrapperName(node);
}

function isNamedWrapper(node) {
  if (node.type === "FunctionDeclaration") {
    return true;
  }
  return Boolean(wrapperName(node));
}

function reportIfAlias(node, context) {
  if (!isNamedWrapper(node)) return;
  const call = onlyCallExpression(node.body);
  if (!call || call.type !== "CallExpression") return;
  if (!directCalleeName(call)) return;
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
