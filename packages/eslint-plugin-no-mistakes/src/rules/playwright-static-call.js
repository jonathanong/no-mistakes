"use strict";

const { literalString } = require("../helpers");
const { patternKey, resolveAlias, variableFor } = require("./playwright-static-call-alias");

const SELECTOR_METHODS = new Set([
  "$",
  "$$",
  "$eval",
  "$$eval",
  "check",
  "click",
  "dblclick",
  "dragAndDrop",
  "fill",
  "focus",
  "frameLocator",
  "getByTestId",
  "hover",
  "locator",
  "press",
  "selectOption",
  "setInputFiles",
  "tap",
  "textContent",
  "type",
  "uncheck",
  "waitForSelector",
]);

function memberName(node) {
  if (!node || node.type !== "MemberExpression") return null;
  if (node.computed && node.property?.type === "Identifier") return null;
  if (!node.computed && typeof node.property?.name === "string") return node.property.name;
  return literalString(node.property);
}

function methodFromCallee(callee, context, seen = new Set()) {
  while (callee?.type === "ChainExpression") callee = callee.expression;
  const direct = memberName(callee);
  if (direct) return direct;
  if (callee?.type !== "Identifier") return null;
  const variable = variableFor(callee, context);
  if (!variable || variable.defs?.length === 0) return callee.name;
  const alias = resolveAlias(callee, context, seen);
  if (!alias) return null;
  if (alias.kind === "key") return alias.key;
  return methodFromCallee(alias.init, context, seen);
}

// A configured factory keeps its own name when that name is also a
// single-write alias of another function. The resolved target is used only
// when the spelled callee is not itself configured.
function configuredFactory(callee, context, factoryNames, seen = new Set()) {
  while (callee?.type === "ChainExpression") callee = callee.expression;
  if (callee?.type === "Identifier" && factoryNames.has(callee.name)) return callee.name;
  const direct = memberName(callee);
  if (direct && factoryNames.has(direct)) return direct;
  if (callee?.type !== "Identifier") return null;
  const alias = resolveAlias(callee, context, seen);
  if (!alias) return null;
  if (alias.kind === "key") return alias.key && factoryNames.has(alias.key) ? alias.key : null;
  return configuredFactory(alias.init, context, factoryNames, seen);
}

function invokedCall(node) {
  if (node?.type !== "CallExpression") return null;
  let callee = node.callee;
  let args = node.arguments || [];
  for (let depth = 0; depth < 4; depth += 1) {
    while (callee?.type === "ChainExpression") callee = callee.expression;
    if (callee?.type === "SequenceExpression" && callee.expressions.length > 0) {
      callee = callee.expressions[callee.expressions.length - 1];
      continue;
    }
    if (callee?.type !== "MemberExpression") break;
    const name = memberName(callee);
    if (name !== "call" && name !== "apply") break;
    if (name === "call") {
      if (args.some((arg) => arg?.type === "SpreadElement")) return null;
      callee = callee.object;
      args = args.slice(1);
      continue;
    }
    const list = args[1]?.type === "ChainExpression" ? args[1].expression : args[1];
    if (
      list?.type !== "ArrayExpression" ||
      list.elements.some((element) => !element || element.type === "SpreadElement")
    ) {
      return null;
    }
    callee = callee.object;
    args = list.elements;
  }
  while (callee?.type === "ChainExpression") callee = callee.expression;
  return { callee, arguments: args };
}

function isNamespaceBinding(node, context, namespaces, seen = new Set()) {
  if (node?.type !== "Identifier") return false;
  const variable = variableFor(node, context);
  if (!variable || variable.defs.length !== 1) return false;
  const def = variable.defs[0];
  if (def.type === "ImportBinding" && def.node.type === "ImportNamespaceSpecifier") {
    return namespaces.has(node.name);
  }
  const alias = resolveAlias(node, context, seen);
  if (alias?.init?.type !== "Identifier") return false;
  return isNamespaceBinding(alias.init, context, namespaces, seen);
}

function isTestBinding(node, context, testNames, seen = new Set()) {
  if (node?.type !== "Identifier") return false;
  const variable = variableFor(node, context);
  if (!variable || variable.defs.length === 0) return testNames.has(node.name);
  const def = variable.defs[0];
  if (def.type === "Parameter") return false;
  if (def.type === "ImportBinding") return testNames.has(node.name);
  const alias = resolveAlias(node, context, seen);
  if (alias?.init?.type !== "Identifier") return false;
  return isTestBinding(alias.init, context, testNames, seen);
}

function isBeforeAllCall(node, testNames, namespaces, context) {
  const invoked = invokedCall(node);
  if (!invoked) return false;
  let callee = invoked.callee;
  while (callee?.type === "ChainExpression") callee = callee.expression;
  if (memberName(callee) !== "beforeAll") return false;
  let object = callee.object;
  while (object?.type === "ChainExpression") object = object.expression;
  if (object?.type === "Identifier") {
    return (
      isTestBinding(object, context, testNames) || isNamespaceBinding(object, context, namespaces)
    );
  }
  if (memberName(object) === "test" && object?.object?.type === "Identifier") {
    return (
      isTestBinding(object.object, context, testNames) ||
      isNamespaceBinding(object.object, context, namespaces)
    );
  }
  return false;
}

module.exports = {
  SELECTOR_METHODS,
  invokedCall,
  isBeforeAllCall,
  memberName,
  configuredFactory,
  methodFromCallee,
  patternKey,
  variableFor,
};
