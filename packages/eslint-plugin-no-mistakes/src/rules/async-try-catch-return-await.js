"use strict";

const { rule } = require("../helpers");
const {
  findContainingFunction,
  isUnconditionalBeforeReturn,
  traverse,
  unwrapExpression,
} = require("./async-ast");
const { handlerOptionsSchema } = require("./async-schema");
const { createTargetMatcher } = require("./async-targets");

function isAwaited(node) {
  return node?.type === "AwaitExpression";
}

function mayReturnPromise(node) {
  const expression = unwrapExpression(node);
  if (expression?.type === "ConditionalExpression") {
    return mayReturnPromise(expression.consequent) || mayReturnPromise(expression.alternate);
  }
  if (expression?.type === "LogicalExpression") {
    return mayReturnPromise(expression.left) || mayReturnPromise(expression.right);
  }
  return expression?.type === "CallExpression" || isPromiseConstructor(expression);
}

function isPromiseConstructor(node) {
  return (
    node?.type === "NewExpression" &&
    node.callee.type === "Identifier" &&
    node.callee.name === "Promise"
  );
}

function variableName(node) {
  return node?.type === "Identifier" ? node.name : null;
}

function resolveVariable(node, context) {
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === node.name);
    if (variable) return variable;
    scope = scope.upper;
  }
}

function isReassignedBeforeReturn(variable, node, allowedWrite, block) {
  const definitionNames = new Set(variable.defs.map((def) => def.name));
  const containingFunction = findContainingFunction(node);
  return variable.references.some(
    (reference) =>
      reference.isWrite() &&
      !definitionNames.has(reference.identifier) &&
      reference.identifier !== allowedWrite &&
      findContainingFunction(reference.identifier) === containingFunction &&
      isUnconditionalBeforeReturn(reference.identifier, block) &&
      reference.identifier.range[0] < node.range[0],
  );
}

function assignedVariable(node, context) {
  if (node.left?.type !== "Identifier" || node.operator !== "=") return null;
  return resolveVariable(node.left, context);
}

function promiseAliasWrite(node, context, promiseAliases) {
  const expression = unwrapExpression(node);
  if (mayReturnPromise(expression)) return node;
  if (expression?.type !== "Identifier") return null;
  const variable = resolveVariable(expression, context);
  return variable ? promiseAliases.get(variable) : null;
}

function returnsPromiseAlias(node, context, promiseAliases, block) {
  const expression = unwrapExpression(node);
  if (expression?.type === "ConditionalExpression") {
    return (
      returnsPromiseAlias(expression.consequent, context, promiseAliases, block) ||
      returnsPromiseAlias(expression.alternate, context, promiseAliases, block)
    );
  }
  if (expression?.type === "LogicalExpression") {
    return (
      returnsPromiseAlias(expression.left, context, promiseAliases, block) ||
      returnsPromiseAlias(expression.right, context, promiseAliases, block)
    );
  }
  if (expression?.type !== "Identifier") return false;
  const variable = resolveVariable(expression, context);
  const promiseWrite = variable ? promiseAliases.get(variable) : null;
  return (
    variable && promiseWrite && !isReassignedBeforeReturn(variable, expression, promiseWrite, block)
  );
}

function shouldParenthesizeAwaitArgument(node) {
  const expression = unwrapExpression(node);
  return (
    expression !== node ||
    expression.type === "ConditionalExpression" ||
    expression.type === "LogicalExpression"
  );
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require return await in configured async try/catch handlers",
      recommended: false,
    },
    hasSuggestions: true,
    schema: handlerOptionsSchema,
    messages: {
      awaitReturn:
        "Use return await inside this try block so rejections are handled by the configured catch handler.",
      addAwait: "Insert await before the returned promise.",
    },
  },
  (context) => {
    const matcher = createTargetMatcher(context, "handlers");
    if (!matcher.hasTargets) return {};

    function catchCallsHandler(catchClause) {
      let matches = false;
      traverse(context, catchClause.body, (node) => {
        if (node.type === "CallExpression" && matcher.isTargetCall(node)) matches = true;
      });
      return matches;
    }

    function reportReturn(node) {
      context.report({
        node,
        messageId: "awaitReturn",
        suggest: [
          {
            messageId: "addAwait",
            fix(fixer) {
              if (shouldParenthesizeAwaitArgument(node.argument)) {
                return fixer.replaceText(
                  node.argument,
                  `await (${context.sourceCode.getText(node.argument)})`,
                );
              }
              return fixer.insertTextBefore(node.argument, "await ");
            },
          },
        ],
      });
    }

    function checkTryBlock(node) {
      if (!node.handler || !catchCallsHandler(node.handler)) return;
      if (!findContainingFunction(node)?.async) return;
      const promiseAliases = new WeakMap();
      traverse(context, node.block, (child) => {
        if (child.type === "VariableDeclarator") {
          const name = variableName(child.id);
          if (!name || isAwaited(child.init)) return;
          const variable = resolveVariable(child.id, context);
          if (variable && promiseAliasWrite(child.init, context, promiseAliases)) {
            promiseAliases.set(variable, child.id);
          }
          return;
        }
        if (child.type === "AssignmentExpression") {
          const variable = assignedVariable(child, context);
          if (!variable) return;
          if (promiseAliasWrite(child.right, context, promiseAliases) && !isAwaited(child.right)) {
            promiseAliases.set(variable, child.left);
          } else if (isUnconditionalBeforeReturn(child, node.block)) {
            promiseAliases.delete(variable);
          }
          return;
        }
        if (child.type === "AwaitExpression") {
          const argument = unwrapExpression(child.argument);
          if (argument.type === "Identifier" && isUnconditionalBeforeReturn(child, node.block)) {
            const variable = resolveVariable(argument, context);
            if (variable) promiseAliases.delete(variable);
          }
          return;
        }
        if (child.type !== "ReturnStatement" || !child.argument || isAwaited(child.argument))
          return;
        const argument = unwrapExpression(child.argument);
        if (
          mayReturnPromise(argument) ||
          returnsPromiseAlias(argument, context, promiseAliases, node.block)
        ) {
          reportReturn(child);
        }
      });
    }

    return {
      ...matcher.visitors,
      "TryStatement:exit": checkTryBlock,
    };
  },
);
