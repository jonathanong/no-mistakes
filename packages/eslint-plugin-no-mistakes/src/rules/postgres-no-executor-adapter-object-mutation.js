"use strict";

const { targetFactsFor } = require("./postgres-no-executor-adapter-facts");
const { tsNonWritingArgument } = require("./postgres-no-executor-adapter-nonwriting");
const { nullishGuardRead } = require("./postgres-no-executor-adapter-guard");
const { mayPrecede } = require("./postgres-no-executor-adapter-mutation-order");
const { primitiveType } = require("./postgres-no-executor-adapter-sql-primitive");

function nonWritingArgument(call, value, context) {
  const services = context?.sourceCode.parserServices;
  if (!services?.program || !services.esTreeNodeToTSNodeMap?.has(call)) return false;
  const index = call.arguments.indexOf(value);
  if (value.type !== "Identifier") return false;
  const ts = require("typescript");
  return tsNonWritingArgument(
    services.esTreeNodeToTSNodeMap.get(call),
    index,
    services.program.getTypeChecker(),
    ts,
    targetFactsFor(context),
    services.program,
  );
}

const LOCAL_WRAPPERS = new Set([
  "TSAsExpression",
  "TSTypeAssertion",
  "TSNonNullExpression",
  "TSSatisfiesExpression",
  "TSInstantiationExpression",
  "ParenthesizedExpression",
  "ChainExpression",
]);

function scalarValue(value, context) {
  const services = context?.sourceCode.parserServices;
  if (!services?.program || !services.esTreeNodeToTSNodeMap?.has(value)) return false;
  const ts = require("typescript");
  const checker = services.program.getTypeChecker();
  const type = checker.getTypeAtLocation(services.esTreeNodeToTSNodeMap.get(value));
  return primitiveType(type, ts, true);
}

function scalarArgument(call, value, context) {
  return call.callee !== value && scalarValue(value, context);
}

function objectMutation(
  variable,
  executorVariable,
  allowedProperty,
  ignoredReference,
  context,
  before = null,
) {
  return variable.references.some((reference) => {
    if (reference.init || reference.identifier === ignoredReference) return false;
    if (!mayPrecede(variable, reference, before)) return false;
    if (before && nullishGuardRead(reference.identifier)) return false;
    let value = reference.identifier;
    let depth = 0;
    while (true) {
      const parent = value.parent;
      if (parent?.expression === value && LOCAL_WRAPPERS.has(parent.type)) value = parent;
      else if (parent?.type === "MemberExpression" && parent.object === value) {
        value = parent;
        depth++;
      } else break;
    }
    const parent = value.parent;
    if (parent?.type === "CallExpression" && parent.arguments.includes(value))
      return !(
        nonWritingArgument(parent, value, context) ||
        (before && depth > 0 && scalarArgument(parent, value, context))
      );
    if (
      (parent?.type === "AssignmentExpression" && parent.left === value) ||
      ((parent?.type === "ForOfStatement" || parent?.type === "ForInStatement") &&
        parent.left === value) ||
      (parent?.type === "UpdateExpression" && parent.argument === value) ||
      (parent?.type === "UnaryExpression" && parent.operator === "delete") ||
      (parent?.type === "VariableDeclarator" &&
        parent.init === value &&
        parent.id.type === "Identifier" &&
        !(before && depth > 0 && scalarValue(value, context))) ||
      (parent?.type === "CallExpression" &&
        parent.callee === value &&
        depth === 1 &&
        (variable !== executorVariable ||
          value.computed ||
          value.property.name !== allowedProperty))
    )
      return true;
    if (depth > 0) return false;
    return !(
      parent?.type === "VariableDeclarator" &&
      parent.init === value &&
      parent.id.type === "ObjectPattern"
    );
  });
}

module.exports = { objectMutation };
