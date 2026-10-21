"use strict";

const { findContainingFunction, unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { propertyName } = require("./module-mock-helpers");
const { hasLaterWrite } = require("./postgres-no-async-sql-builder-bindings");

function arrayLiteralMap(call) {
  const callee = unwrapExpression(call.callee);
  if (callee?.type !== "MemberExpression" || callee.computed || callee.optional) return false;
  if (propertyName(callee.property) !== "map") return false;
  return unwrapExpression(callee.object)?.type === "ArrayExpression";
}

// `[1].map(() => build())` is an array of promises. Returning that array from
// an async function still lets a caller observe each element.
function arrayMapValue(node, context, seen = new Set()) {
  const value = unwrapExpression(node);
  if (!value || seen.has(value)) return false;
  seen.add(value);
  if (value.type === "AwaitExpression") return arrayMapValue(value.argument, context, seen);
  if (value.type === "SequenceExpression") {
    return arrayMapValue(value.expressions.at(-1), context, seen);
  }
  if (value.type === "ConditionalExpression" || value.type === "LogicalExpression") {
    return (
      arrayMapValue(value.consequent ?? value.left, context, seen) ||
      arrayMapValue(value.alternate ?? value.right, context, seen)
    );
  }
  if (value.type === "Identifier") return bindingMapValue(value, context, seen);
  return value.type === "CallExpression" && arrayLiteralMap(value);
}

function bindingMapValue(identifier, context, seen) {
  const variable = resolveVariable(identifier, context);
  if (!variable || variable.defs.length !== 1) return false;
  const definition = variable.defs[0];
  if (definition.type !== "Variable" || hasLaterWrite(variable)) return false;
  return arrayMapValue(definition.node.init, context, seen);
}

function returnsAsyncArrayMap(returnStatement, value, context) {
  return findContainingFunction(returnStatement).async && arrayMapValue(value, context);
}

module.exports = {
  arrayLiteralMap,
  arrayMapValue,
  returnsAsyncArrayMap,
};
