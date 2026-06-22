"use strict";

const { expressionName, literalString, memberPropertyName } = require("./module-mock-helpers");
const { resolveVariable } = require("./module-mock-preserve-aliases");

function factoryParamName(factory) {
  const param = factory?.params?.[0];
  const unwrapped = param?.type === "AssignmentPattern" ? param.left : param;
  return unwrapped?.type === "Identifier" ? unwrapped.name : undefined;
}

function unwrapExpression(node) {
  let current = node;
  while (
    current?.type === "TSAsExpression" ||
    current?.type === "TSSatisfiesExpression" ||
    current?.type === "TSNonNullExpression" ||
    current?.type === "TypeCastExpression"
  ) {
    current = current.expression;
  }
  return current;
}

function isFunctionNode(node) {
  return (
    node?.type === "FunctionDeclaration" ||
    node?.type === "FunctionExpression" ||
    node?.type === "ArrowFunctionExpression"
  );
}

function realModuleCall(call, specifier, paramName, mock) {
  if (call?.type !== "CallExpression") return false;
  if (call.callee.type === "MemberExpression") {
    const objectName = expressionName(call.callee.object);
    const prop = memberPropertyName(call.callee);
    if (literalString(call.arguments[0]) !== specifier) return false;
    if (prop === "importActual" && mock.framework === "vitest" && objectName === mock.namespace) {
      return "async";
    }
    if (
      prop === "requireActual" &&
      mock.framework === "@jest/globals" &&
      objectName === mock.namespace
    ) {
      return "sync";
    }
    return false;
  }
  if (call.callee.type === "Identifier" && paramName && call.callee.name === paramName) {
    return "async";
  }
  return false;
}

function spreadArgumentPreserves(argument, specifier, paramName, mock) {
  argument = unwrapExpression(argument);
  if (argument?.type === "AwaitExpression") {
    return Boolean(realModuleCall(argument.argument, specifier, paramName, mock));
  }
  return realModuleCall(argument, specifier, paramName, mock) === "sync";
}

function collectRealModuleNames(node, realModuleNames, specifier, paramName, mock, context) {
  if (isFunctionNode(node)) return;
  if (node.type === "VariableDeclaration" && node.kind === "const") {
    for (const declarator of node.declarations) {
      if (
        declarator.id.type === "Identifier" &&
        spreadArgumentPreserves(declarator.init, specifier, paramName, mock)
      ) {
        realModuleNames.add(resolveVariable(declarator.id, context));
      }
    }
  }
  for (const key of ["block", "body", "consequent", "alternate", "finalizer", "handler"]) {
    const child = node[key];
    if (Array.isArray(child)) {
      for (const item of child)
        collectRealModuleNames(item, realModuleNames, specifier, paramName, mock, context);
    } else if (child?.type) {
      collectRealModuleNames(child, realModuleNames, specifier, paramName, mock, context);
    }
  }
}

function collectReturnedObjects(node, objects) {
  if (isFunctionNode(node)) return;
  if (node.type === "ReturnStatement") {
    const argument = unwrapExpression(node.argument);
    objects.push(argument?.type === "ObjectExpression" ? argument : null);
    return;
  }
  for (const key of ["block", "body", "consequent", "alternate", "finalizer", "handler"]) {
    const child = node[key];
    if (Array.isArray(child)) {
      for (const item of child) collectReturnedObjects(item, objects);
    } else if (child?.type) {
      collectReturnedObjects(child, objects);
    }
  }
}

function analyzeFactoryBody(factory, specifier, paramName, mock, context) {
  const body = unwrapExpression(factory?.body);
  const objects = [];
  const realModuleNames = new Set();
  if (!body) return { objects, realModuleNames };
  if (body.type === "ObjectExpression") return { objects: [body], realModuleNames };
  if (body.type !== "BlockStatement") return { objects, realModuleNames };

  for (const statement of body.body) {
    collectRealModuleNames(statement, realModuleNames, specifier, paramName, mock, context);
    collectReturnedObjects(statement, objects);
  }
  return { objects, realModuleNames };
}

function objectSpreadsRealModule(object, specifier, paramName, realModuleNames, mock, context) {
  return object.properties.some((prop) => {
    if (prop.type !== "SpreadElement") return false;
    if (prop.argument.type === "Identifier") {
      return realModuleNames.has(resolveVariable(prop.argument, context));
    }
    return spreadArgumentPreserves(prop.argument, specifier, paramName, mock);
  });
}

function factoryPreservesExports(factory, specifier, mock, context) {
  if (!isFunctionNode(factory)) return false;
  const paramName = mock.framework === "vitest" ? factoryParamName(factory) : undefined;
  const { objects, realModuleNames } = analyzeFactoryBody(
    factory,
    specifier,
    paramName,
    mock,
    context,
  );
  return (
    objects.length > 0 &&
    objects.every(
      (object) =>
        object &&
        objectSpreadsRealModule(object, specifier, paramName, realModuleNames, mock, context),
    )
  );
}

module.exports = { factoryPreservesExports };
