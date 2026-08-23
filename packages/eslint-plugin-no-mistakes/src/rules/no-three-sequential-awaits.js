"use strict";

const { rule } = require("../helpers");

function unwrapExpression(node) {
  let current = node;
  while (
    current &&
    ["ChainExpression", "TSAsExpression", "TSNonNullExpression", "TSTypeAssertion"].includes(
      current.type,
    )
  ) {
    current = current.expression;
  }
  return current;
}

function isPromiseAllCall(node) {
  const callee = unwrapExpression(node)?.callee;
  if (!callee || callee.type !== "MemberExpression" || callee.computed) return false;
  const object = unwrapExpression(callee.object);
  const property = callee.property;
  return (
    object?.type === "Identifier" &&
    object.name === "Promise" &&
    property?.type === "Identifier" &&
    (property.name === "all" || property.name === "allSettled")
  );
}

function awaitedArgument(statement) {
  if (statement.type === "ExpressionStatement") {
    const expression = unwrapExpression(statement.expression);
    if (expression?.type === "AwaitExpression") return expression.argument;
    if (expression?.type === "AssignmentExpression") {
      const right = unwrapExpression(expression.right);
      if (right?.type === "AwaitExpression") return right.argument;
    }
    return null;
  }
  if (statement.type !== "VariableDeclaration" || statement.declarations.length !== 1) {
    return null;
  }
  const init = unwrapExpression(statement.declarations[0].init);
  return init?.type === "AwaitExpression" ? init.argument : null;
}

function isCountedAwait(statement) {
  const argument = awaitedArgument(statement);
  return Boolean(argument) && !isPromiseAllCall(argument);
}

function scanBlock(statements, context) {
  let run = 0;
  for (const statement of statements) {
    if (!isCountedAwait(statement)) {
      run = 0;
      continue;
    }
    run += 1;
    if (run >= 3) context.report({ node: statement, messageId: "sequential" });
  }
}

module.exports = rule(
  {
    type: "suggestion",
    docs: {
      description: "disallow three sequential await statements",
      recommended: false,
    },
    schema: [],
    messages: {
      sequential:
        "Avoid 3 sequential await statements; abstract dependent work or parallelize independent work with Promise.all().",
    },
  },
  (context) => ({
    BlockStatement(node) {
      scanBlock(node.body, context);
    },
    Program(node) {
      scanBlock(node.body, context);
    },
    SwitchCase(node) {
      scanBlock(node.consequent, context);
    },
  }),
);
