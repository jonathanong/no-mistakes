"use strict";

const { rule } = require("../helpers");
const { createTargetMatcher } = require("./async-targets");

function isFunction(node) {
  return (
    node?.type === "ArrowFunctionExpression" ||
    node?.type === "FunctionDeclaration" ||
    node?.type === "FunctionExpression"
  );
}

function unwrapExpression(node) {
  let current = node;
  while (
    current?.type === "ChainExpression" ||
    current?.type === "TSNonNullExpression" ||
    current?.type === "TSAsExpression" ||
    current?.type === "TSTypeAssertion" ||
    current?.type === "TSSatisfiesExpression"
  ) {
    current = current.expression;
  }
  return current;
}

function isAwaited(node) {
  return node?.type === "AwaitExpression";
}

function mayReturnPromise(node) {
  const expression = unwrapExpression(node);
  if (expression?.type === "ConditionalExpression") {
    return mayReturnPromise(expression.consequent) || mayReturnPromise(expression.alternate);
  }
  return (
    expression?.type === "CallExpression" ||
    expression?.type === "NewExpression" ||
    expression?.type === "ImportExpression"
  );
}

function findContainingFunction(node) {
  let current = node.parent;
  while (current) {
    if (isFunction(current)) return current;
    current = current.parent;
  }
}

function visitorKeys(context, node) {
  return context.sourceCode.visitorKeys[node.type] || [];
}

function traverse(context, node, visit, root = node) {
  if (!node) return;
  if (node !== root && isFunction(node)) return;
  visit(node);
  for (const key of visitorKeys(context, node)) {
    const value = node[key];
    if (Array.isArray(value)) {
      for (const child of value) {
        if (child?.type) traverse(context, child, visit, root);
      }
    } else if (value?.type) {
      traverse(context, value, visit, root);
    }
  }
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

function isReassignedBeforeReturn(variable, node) {
  const definitionNames = new Set(variable.defs.map((def) => def.name));
  return variable.references.some(
    (reference) =>
      reference.isWrite() &&
      !definitionNames.has(reference.identifier) &&
      reference.identifier.range[0] < node.range[0],
  );
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require return await in configured async try/catch handlers",
      recommended: false,
    },
    fixable: "code",
    schema: [
      {
        type: "object",
        properties: {
          targets: {
            type: "array",
            items: {
              type: "object",
              properties: {
                sourcePatterns: { type: "array", items: { type: "string" } },
                calleeNamePatterns: { type: "array", items: { type: "string" } },
              },
              additionalProperties: false,
            },
          },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      awaitReturn:
        "Use return await inside this try block so rejections are handled by the configured catch handler.",
    },
  },
  (context) => {
    const matcher = createTargetMatcher(context);
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
        fix(fixer) {
          if (unwrapExpression(node.argument) !== node.argument) {
            return fixer.replaceText(
              node.argument,
              `await (${context.sourceCode.getText(node.argument)})`,
            );
          }
          return fixer.insertTextBefore(node.argument, "await ");
        },
      });
    }

    function checkTryBlock(node) {
      if (!node.handler || !catchCallsHandler(node.handler)) return;
      if (!findContainingFunction(node)?.async) return;
      const promiseAliases = new WeakSet();
      traverse(context, node.block, (child) => {
        if (child.type === "VariableDeclarator") {
          const name = variableName(child.id);
          if (!name || isAwaited(child.init)) return;
          const variable = resolveVariable(child.id, context);
          if (variable && mayReturnPromise(child.init)) promiseAliases.add(variable);
          return;
        }
        if (child.type !== "ReturnStatement" || !child.argument || isAwaited(child.argument))
          return;
        const argument = unwrapExpression(child.argument);
        if (mayReturnPromise(argument)) {
          reportReturn(child);
          return;
        }
        if (argument.type === "Identifier") {
          const variable = resolveVariable(argument, context);
          if (
            variable &&
            promiseAliases.has(variable) &&
            !isReassignedBeforeReturn(variable, argument)
          ) {
            reportReturn(child);
          }
        }
      });
    }

    return {
      ...matcher.visitors,
      TryStatement: checkTryBlock,
    };
  },
);
