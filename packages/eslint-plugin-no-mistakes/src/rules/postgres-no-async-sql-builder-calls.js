"use strict";

const { isFunction, isTransparentExpression, unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { propertyName } = require("./module-mock-helpers");
const {
  bindingResultContract,
  boundVariable,
  exportContract,
  hasLaterWrite,
  returnedPromiseContract,
} = require("./postgres-no-async-sql-builder-bindings");
const { callableReturnContract } = require("./postgres-no-async-sql-builder-callable");
const { annotationContract, combine } = require("./postgres-no-async-sql-builder-types");

const PROMISE_USES = new Set(["then", "catch", "finally"]);

function isThenUse(node) {
  const member = node.parent;
  if (member?.type !== "MemberExpression" || member.object !== node || member.computed)
    return false;
  const name = propertyName(member.property);
  return (
    PROMISE_USES.has(name) &&
    member.parent?.type === "CallExpression" &&
    member.parent.callee === member
  );
}

function callResultContract(call, context, types, seen = new Set()) {
  let current = call;
  while (current) {
    if (isThenUse(current)) return "requires";
    const parent = current.parent;
    if (parent.type === "ExpressionStatement") return "allows";
    if (
      parent.type === "VariableDeclarator" &&
      parent.init === current &&
      parent.id.type === "Identifier"
    ) {
      return bindingResultContract(parent, context, types, seen);
    }
    if (parent.type === "AwaitExpression" && parent.argument === current) return "allows";
    if (parent.type === "ReturnStatement" && parent.argument === current)
      return returnedPromiseContract(parent, context, types);
    if (parent.type === "ArrowFunctionExpression" && parent.body === current) {
      // `() => build()` returns the promise. A discarded `[1].map` result does
      // not observe it; a typed callback parameter still can.
      return conciseBodyContract(parent, context, types, seen);
    }
    if (parent.type === "CallExpression" && parent.arguments.includes(current)) {
      // This value is the builder's Promise, not the builder function.
      return argumentParameterContract(current, context, types, annotationContract);
    }
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
      // A Promise is always truthy, so the branch can change once async is
      // removed, and the conditional throws the SQL away.
      if (parent.type === "ConditionalExpression" && parent.test === current) return "unknown";
      if (parent.type === "LogicalExpression" && parent.left === current) return "unknown";
      current = parent;
      continue;
    }
    return "unknown";
  }
}

function argumentParameterContract(argument, context, types, contractOf) {
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
  // A missing annotation is not proof. `annotationContract` treats that as
  // a non-Promise, which would suggest a rewrite the call may not survive.
  if (binding?.type !== "Identifier" || !binding.typeAnnotation) return "unknown";
  return contractOf(binding.typeAnnotation, types);
}

function conciseBodyContract(func, context, types, seen) {
  if (func.async) return "allows";
  if (func.returnType) return annotationContract(func.returnType, types);
  const parent = func.parent;
  if (parent?.type !== "CallExpression" || !parent.arguments.includes(func)) return "unknown";
  if (unwrapExpression(parent.callee)?.type === "Identifier") {
    return argumentParameterContract(func, context, types, callableReturnContract);
  }
  return callResultContract(parent, context, types, seen);
}

function referenceUseContract(identifier, context, types) {
  const parent = identifier.parent;
  if (!parent || parent.type === "ExportSpecifier") return "unknown";
  if (parent.type === "CallExpression" && parent.callee === identifier)
    return callResultContract(parent, context, types);
  if (parent.type === "CallExpression" && parent.arguments.includes(identifier)) {
    return argumentParameterContract(identifier, context, types, callableReturnContract);
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
  if (parent?.type === "CallExpression" && parent.arguments.includes(func)) {
    return argumentParameterContract(func, context, types, callableReturnContract);
  }
  return "unknown";
}

// Class fields, methods, and object properties abstain. Heritage and object
// escape are not modeled.
function builderIsMember(func) {
  const parent = func.parent;
  return (
    parent.type === "MethodDefinition" ||
    parent.type === "PropertyDefinition" ||
    parent.type === "Property"
  );
}

function contractFor(func, context, types) {
  if (builderIsMember(func)) return "unknown";
  return combine([
    annotationContract(func.returnType, types),
    exportContract(func, context),
    referenceContract(func, context, types),
  ]);
}

module.exports = {
  callResultContract,
  contractFor,
  referenceContract,
};
