"use strict";

const {
  frameworkBindingModule,
  literalString,
  memberPropertyName,
} = require("./module-mock-helpers");
const { resolveVariable } = require("./module-mock-preserve-aliases");

function factoryParam(factory) {
  const param = factory?.params?.[0];
  const unwrapped = param?.type === "AssignmentPattern" ? param.left : param;
  return unwrapped?.type === "Identifier" ? unwrapped : null;
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

function realModuleCall(call, specifier, paramVariable, mock, context) {
  if (call?.type !== "CallExpression") return false;
  if (call.callee.type === "MemberExpression") {
    const framework = frameworkBindingModule(call.callee.object, context);
    const prop = memberPropertyName(call.callee);
    if (literalString(call.arguments[0]) !== specifier) return false;
    if (prop === "importActual" && mock?.framework === "vitest" && framework === "vitest") {
      return "async";
    }
    if (
      prop === "requireActual" &&
      mock?.framework === "@jest/globals" &&
      framework === "@jest/globals"
    ) {
      return "sync";
    }
    return false;
  }
  if (
    call.callee.type === "Identifier" &&
    paramVariable &&
    resolveVariable(call.callee, context) === paramVariable
  ) {
    return "async";
  }
  return false;
}

function spreadArgumentPreserves(argument, specifier, paramVariable, mock, context) {
  argument = unwrapExpression(argument);
  if (argument?.type === "AwaitExpression") {
    return Boolean(realModuleCall(argument.argument, specifier, paramVariable, mock, context));
  }
  return realModuleCall(argument, specifier, paramVariable, mock, context) === "sync";
}

function collectRealModuleNames(node, realModuleNames, specifier, paramVariable, mock, context) {
  if (isFunctionNode(node)) return;
  if (node.type === "VariableDeclaration" && node.kind === "const") {
    for (const declarator of node.declarations) {
      if (
        declarator.id.type === "Identifier" &&
        spreadArgumentPreserves(declarator.init, specifier, paramVariable, mock, context)
      ) {
        realModuleNames.add(resolveVariable(declarator.id, context));
      }
    }
  }
  for (const key of ["block", "body", "cases", "consequent", "alternate", "finalizer", "handler"]) {
    const child = node[key];
    if (Array.isArray(child)) {
      for (const item of child)
        collectRealModuleNames(item, realModuleNames, specifier, paramVariable, mock, context);
    } else if (child?.type) {
      collectRealModuleNames(child, realModuleNames, specifier, paramVariable, mock, context);
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
  for (const key of ["block", "body", "cases", "consequent", "alternate", "finalizer", "handler"]) {
    const child = node[key];
    if (Array.isArray(child)) {
      for (const item of child) collectReturnedObjects(item, objects);
    } else if (child?.type) {
      collectReturnedObjects(child, objects);
    }
  }
}

function analyzeFactory(factory, specifier, mock, context) {
  const param = mock?.framework === "vitest" ? factoryParam(factory) : null;
  const paramVariable = param ? resolveVariable(param, context) : null;
  const body = unwrapExpression(factory?.type === "ObjectExpression" ? factory : factory?.body);
  const objects = [];
  const realModuleNames = new Set();
  const analysis = { objects, paramVariable, realModuleNames };
  if (!body) return analysis;
  if (body.type === "ObjectExpression") return { ...analysis, objects: [body] };
  if (body.type !== "BlockStatement") return analysis;

  for (const statement of body.body) {
    collectRealModuleNames(statement, realModuleNames, specifier, paramVariable, mock, context);
    collectReturnedObjects(statement, objects);
  }
  return analysis;
}

function spreadPreservesRealModule(prop, analysis, specifier, mock, context) {
  if (prop.type !== "SpreadElement") return false;
  const { paramVariable, realModuleNames } = analysis;
  if (prop.argument.type === "Identifier") {
    return realModuleNames.has(resolveVariable(prop.argument, context));
  }
  return spreadArgumentPreserves(prop.argument, specifier, paramVariable, mock, context);
}

function objectSpreadsRealModule(object, analysis, specifier, mock, context) {
  return object.properties.some((prop) => {
    return spreadPreservesRealModule(prop, analysis, specifier, mock, context);
  });
}

function factoryPreservesExports(factory, specifier, mock, context) {
  if (!isFunctionNode(factory)) return false;
  const analysis = analyzeFactory(factory, specifier, mock, context);
  const { objects } = analysis;
  return (
    objects.length > 0 &&
    objects.every(
      (object) => object && objectSpreadsRealModule(object, analysis, specifier, mock, context),
    )
  );
}

module.exports = { analyzeFactory, factoryPreservesExports, spreadPreservesRealModule };
