"use strict";

const { isFunction, isTransparentExpression, unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const {
  bindingResultContract,
  boundVariable,
  hasLaterWrite,
  isThenCall,
  returnedPromiseContract,
} = require("./postgres-no-async-sql-builder-bindings");
const { callableReturnContract } = require("./postgres-no-async-sql-builder-property");
const { combine } = require("./postgres-no-async-sql-builder-types");

function callResultContract(call, context, types) {
  let current = call;
  while (current) {
    if (isThenCall(current)) return "requires";
    const parent = current.parent;
    if (!parent) return "unknown";
    if (parent.type === "ExpressionStatement") return "allows";
    if (
      parent.type === "VariableDeclarator" &&
      parent.init === current &&
      parent.id.type === "Identifier"
    ) {
      return bindingResultContract(parent, context, types);
    }
    if (parent.type === "AwaitExpression" && parent.argument === current) return "allows";
    if (parent.type === "ReturnStatement" && parent.argument === current)
      return returnedPromiseContract(parent, context, types);
    if (parent.type === "CallExpression" && parent.arguments.includes(current)) return "allows";
    if (
      parent.type === "UnaryExpression" &&
      parent.operator === "void" &&
      parent.argument === current
    )
      return "allows";
    if (
      parent.type === "ChainExpression" ||
      isTransparentExpression(parent) ||
      parent.type === "ConditionalExpression" ||
      parent.type === "LogicalExpression" ||
      parent.type === "SequenceExpression"
    ) {
      current = parent;
      continue;
    }
    return "unknown";
  }
  return "unknown";
}

function argumentParameterContract(argument, context, types) {
  const call = argument.parent;
  const index = call.arguments.indexOf(argument);
  const callee = unwrapExpression(call.callee);
  if (callee?.type !== "Identifier") return "unknown";
  const variable = resolveVariable(callee, context);
  if (!variable || variable.defs.length !== 1) return "unknown";
  const definition = variable.defs[0];
  const target =
    definition.type === "FunctionName"
      ? definition.node
      : definition.type === "Variable" && isFunction(definition.node.init)
        ? definition.node.init
        : null;
  const parameter = target?.params?.[index];
  if (!parameter || parameter.type === "RestElement") return "unknown";
  const binding = parameter.type === "AssignmentPattern" ? parameter.left : parameter;
  return binding?.type === "Identifier"
    ? callableReturnContract(binding.typeAnnotation, types)
    : "unknown";
}

function referenceUseContract(identifier, context, types) {
  const parent = identifier.parent;
  if (!parent || parent.type === "ExportSpecifier") return "unknown";
  if (parent.type === "CallExpression" && parent.callee === identifier)
    return callResultContract(parent, context, types);
  if (parent.type === "CallExpression" && parent.arguments.includes(identifier)) {
    return argumentParameterContract(identifier, context, types);
  }
  return "unknown";
}

function referenceContract(func, context, types) {
  const variable = boundVariable(func, context);
  if (!variable) return directValueContract(func, context, types);
  if (hasLaterWrite(variable, func.id)) return "unknown";
  const results = ["allows"];
  for (const reference of variable.references) {
    if (reference.init || reference.identifier === func.id) continue;
    results.push(referenceUseContract(reference.identifier, context, types));
  }
  return combine(results);
}

function directValueContract(func, context, types) {
  const parent = func.parent;
  if (
    parent?.type === "MethodDefinition" ||
    parent?.type === "PropertyDefinition" ||
    parent?.type === "Property" ||
    (parent?.type === "VariableDeclarator" && parent.init === func)
  ) {
    return "allows";
  }
  if (parent?.type === "CallExpression" && parent.arguments.includes(func)) {
    return argumentParameterContract(func, context, types);
  }
  return "unknown";
}

module.exports = {
  callResultContract,
  referenceContract,
};
