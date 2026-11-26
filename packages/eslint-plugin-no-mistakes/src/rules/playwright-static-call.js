"use strict";

const { literalString } = require("../helpers");
const { unwrapExpression } = require("./async-ast");
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
  const target = unwrapExpression(node);
  if (!target || target.type !== "MemberExpression") return null;
  const property = unwrapExpression(target.property);
  if (target.computed && property?.type === "Identifier") return null;
  if (!target.computed && typeof target.property?.name === "string") return target.property.name;
  return literalString(property);
}

function methodFromCallee(callee, context, seen = new Set()) {
  callee = unwrapExpression(callee);
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
  callee = unwrapExpression(callee);
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
    callee = unwrapExpression(callee);
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
    const list = unwrapExpression(args[1]);
    if (
      list?.type !== "ArrayExpression" ||
      list.elements.some((element) => !element || element.type === "SpreadElement")
    ) {
      return null;
    }
    callee = callee.object;
    args = list.elements;
  }
  callee = unwrapExpression(callee);
  return { callee, arguments: args };
}

function isNamespaceBinding(node, context, namespaces, seen = new Set()) {
  node = unwrapExpression(node);
  if (node?.type !== "Identifier") return false;
  const variable = variableFor(node, context);
  if (!variable || variable.defs.length !== 1) return false;
  const def = variable.defs[0];
  if (def.type === "ImportBinding" && def.node.type === "ImportNamespaceSpecifier") {
    return namespaces.has(node.name);
  }
  const alias = resolveAlias(node, context, seen);
  const init = unwrapExpression(alias?.init);
  if (init?.type !== "Identifier") return false;
  return isNamespaceBinding(init, context, namespaces, seen);
}

function isTestBinding(node, context, testNames, seen = new Set()) {
  node = unwrapExpression(node);
  if (node?.type !== "Identifier") return false;
  const variable = variableFor(node, context);
  if (!variable || variable.defs.length === 0) return testNames.has(node.name);
  const def = variable.defs[0];
  if (def.type === "Parameter") return false;
  if (def.type === "ImportBinding") return testNames.has(node.name);
  const alias = resolveAlias(node, context, seen);
  const init = unwrapExpression(alias?.init);
  if (init?.type !== "Identifier") return false;
  return isTestBinding(init, context, testNames, seen);
}

function isBeforeAllCall(node, testNames, namespaces, context) {
  const invoked = invokedCall(node);
  if (!invoked) return false;
  const callee = unwrapExpression(invoked.callee);
  if (memberName(callee) !== "beforeAll") return false;
  const object = unwrapExpression(callee.object);
  if (object?.type === "Identifier") {
    return (
      isTestBinding(object, context, testNames) || isNamespaceBinding(object, context, namespaces)
    );
  }
  const parent = unwrapExpression(object?.object);
  if (memberName(object) === "test" && parent?.type === "Identifier") {
    return (
      isTestBinding(parent, context, testNames) || isNamespaceBinding(parent, context, namespaces)
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
