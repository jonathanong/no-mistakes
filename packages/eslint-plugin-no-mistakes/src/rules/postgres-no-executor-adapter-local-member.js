"use strict";

const { resolveVariable, unwrapTs } = require("./postgres-query-text");
const {
  constInitializer,
  containsDirectEval,
  containsReceiverAccess,
} = require("./postgres-no-executor-adapter-binding");
const { propertyText } = require("./postgres-no-executor-adapter-logic");

function staticHelperName(callee) {
  const call = callee.parent;
  if (
    callee.type !== "MemberExpression" ||
    callee.optional ||
    callee.object.type !== "Identifier" ||
    call?.type !== "CallExpression" ||
    call.callee !== callee ||
    call.optional
  )
    return null;
  return propertyText(callee);
}

function initializerKey(property) {
  if (property.type !== "Property" || property.kind !== "init") return null;
  if (property.computed) return propertyText({ computed: true, property: property.key });
  const key = property.key;
  if (key.type === "Identifier") return key.name;
  return key.type === "Literal" && ["string", "number"].includes(typeof key.value)
    ? String(key.value)
    : null;
}

function closedReceiverTargets(variable, receiver, context, facts) {
  const object = unwrapTs(constInitializer(receiver, context, new Set()));
  if (object?.type !== "ObjectExpression") return null;
  const declaration = variable.defs[0].parent;
  if (declaration.parent?.type === "ExportNamedDeclaration") return null;

  const methods = new Map();
  for (const property of object.properties) {
    const key = initializerKey(property);
    if (key === null || key === "__proto__" || methods.has(key)) return null;
    const fn = unwrapTs(property.value);
    methods.set(
      key,
      fn?.type === "FunctionExpression" || fn?.type === "ArrowFunctionExpression" ? fn : null,
    );
  }

  const targets = new WeakMap();
  const usedMethods = new Set();
  for (const reference of variable.references) {
    if (reference.init) continue;
    const id = reference.identifier;
    const member = id.parent;
    if (reference.tainted || reference.isWrite() || member?.object !== id) return null;
    const name = staticHelperName(member);
    const fn = methods.get(name);
    if (!fn) return null;
    targets.set(member, fn);
    usedMethods.add(fn);
  }
  // Nested class heritage and computed keys can use the enclosing receiver.
  for (const fn of usedMethods) {
    if (containsReceiverAccess(fn)) return null;
  }

  const evalByScope = (facts.localMemberEval ??= new WeakMap());
  const scope = variable.scope.block;
  if (!evalByScope.has(scope)) evalByScope.set(scope, containsDirectEval(scope));
  return evalByScope.get(scope) ? null : targets;
}

function localMemberFunction(callee, context, facts) {
  if (staticHelperName(callee) === null) return null;
  const receiver = callee.object;
  const variable = resolveVariable(receiver, context);
  if (!variable) return null;
  const byReceiver = (facts.localMemberTargets ??= new WeakMap());
  if (!byReceiver.has(variable)) {
    byReceiver.set(variable, closedReceiverTargets(variable, receiver, context, facts));
  }
  return byReceiver.get(variable)?.get(callee) ?? null;
}

module.exports = { localMemberFunction, staticHelperName };
