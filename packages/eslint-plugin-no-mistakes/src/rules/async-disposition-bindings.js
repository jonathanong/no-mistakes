"use strict";

const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { resolveVariable, stableConstInitializer } = require("./async-target-bindings");
const {
  collectionAnchor,
  laterDrain,
  laterInBlock,
  statement,
} = require("./async-disposition-flow");
const { propertiesWritten } = require("./async-disposition-mocks");
const { memberPropertyName } = require("./async-targets");

function globalPromise(node, context) {
  const variable = resolveVariable(node, context);
  return !variable?.defs.length && !propertiesWritten(node, context);
}

function isPromiseMethodCall(node, context, methods) {
  const callee = unwrapExpression(node?.callee);
  const object = unwrapExpression(callee?.object);
  return (
    node?.type === "CallExpression" &&
    callee?.type === "MemberExpression" &&
    object?.type === "Identifier" &&
    object.name === "Promise" &&
    globalPromise(object, context) &&
    methods.includes(memberPropertyName(callee))
  );
}

function isPromiseAllCall(node, context) {
  return isPromiseMethodCall(node, context, ["all", "allSettled", "race", "any"]);
}

function standalonePush(call) {
  return (
    call.parent.type === "ExpressionStatement" ||
    (call.parent.type === "ArrowFunctionExpression" && call.parent.body === call)
  );
}

function pushedArray(node) {
  const value = unwrapTransparentParent(node);
  const push = value.parent;
  const callee = unwrapExpression(push?.callee);
  if (
    push?.type !== "CallExpression" ||
    !push.arguments.includes(value) ||
    callee?.type !== "MemberExpression" ||
    memberPropertyName(callee) !== "push" ||
    callee.object.type !== "Identifier" ||
    !standalonePush(push)
  )
    return null;
  return { id: callee.object, push };
}

function collectedPromise(node, context, isAggregate, disposed) {
  const found = pushedArray(node);
  if (!found) return false;
  const init = unwrapExpression(stableConstInitializer(found.id, context, new Set()));
  if (init?.type !== "ArrayExpression" || init.elements.length !== 0) return false;
  const variable = resolveVariable(found.id, context);
  let drained = false;
  for (const reference of variable.references) {
    if (reference.init) continue;
    const id = reference.identifier;
    const parent = unwrapTransparentParent(id).parent;
    if (isAggregate(parent) && parent.arguments[0] === id) {
      if (laterDrain(found.push, parent) && disposed(parent)) drained = true;
      continue;
    }
    const member = parent;
    const call = member?.parent;
    if (
      member?.type !== "MemberExpression" ||
      member.object !== id ||
      memberPropertyName(member) !== "push" ||
      call?.type !== "CallExpression" ||
      call.callee !== member ||
      !standalonePush(call) ||
      !collectionAnchor(call) ||
      statement(collectionAnchor(call)).parent !==
        statement(collectionAnchor(found.push) || found.push).parent
    )
      return false;
  }
  return drained;
}

function retainedCollection(node, context, isAggregate, disposed) {
  const expression = unwrapTransparentParent(node);
  const declaration = expression.parent;
  if (
    declaration?.type !== "VariableDeclarator" ||
    declaration.init !== expression ||
    declaration.id.type !== "Identifier" ||
    !stableConstInitializer(declaration.id, context, new Set())
  )
    return false;
  const variable = resolveVariable(declaration.id, context);
  let drained = false;
  for (const reference of variable.references) {
    if (reference.init) continue;
    const id = unwrapTransparentParent(reference.identifier);
    const parent = id.parent;
    if (!isAggregate(parent) || parent.arguments[0] !== id) return false;
    if (laterDrain(node, parent) && disposed(parent)) drained = true;
  }
  return drained;
}

function retainedPromise(node, context, disposed) {
  const expression = unwrapTransparentParent(node);
  const declaration = expression.parent;
  if (
    declaration?.type !== "VariableDeclarator" ||
    declaration.init !== expression ||
    declaration.id.type !== "Identifier"
  )
    return false;
  if (!stableConstInitializer(declaration.id, context, new Set())) return false;
  const variable = resolveVariable(declaration.id, context);
  return variable.references.some(
    (reference) =>
      !reference.init && laterInBlock(node, reference.identifier) && disposed(reference.identifier),
  );
}

module.exports = {
  collectedPromise,
  isPromiseAllCall,
  isPromiseMethodCall,
  retainedCollection,
  retainedPromise,
};
