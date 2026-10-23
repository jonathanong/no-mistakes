"use strict";

const { lookup } = require("./no-object-mutation-values");
const { methodName, unwrap } = require("./no-object-mutation-ast");

const { ARRAY_METHODS } = require("./no-object-mutation-ast");

const OBJECT_METHODS = new Set(["assign", "defineProperties", "defineProperty", "setPrototypeOf"]);
const REFLECT_METHODS = new Set(["defineProperty", "set"]);

function isGlobal(node, scope, name) {
  const target = unwrap(node);
  return Boolean(target?.type === "Identifier" && target.name === name && !lookup(scope, name));
}

function firstArgument(node) {
  const target = node.arguments[0];
  if (!target || target.type === "SpreadElement") {
    return null;
  }
  return target;
}

function argumentMutation(node) {
  const target = firstArgument(node);
  if (target) {
    return { targetNode: target, otherArgs: node.arguments.slice(1) };
  }
  return null;
}

function objectMutation(node, scope, object, name) {
  if (!OBJECT_METHODS.has(name) || !isGlobal(object, scope, "Object")) {
    return null;
  }
  return argumentMutation(node);
}

// Array `.call`/`.apply` mutate the first argument. Object statics ignore
// `this` and mutate their own first argument, which follows `thisArg`.
function forwardedMutation(receiver, node, scope) {
  const method = unwrap(receiver);
  if (method.type !== "MemberExpression") {
    return null;
  }
  const name = methodName(method);
  if (!name) {
    return null;
  }
  if (ARRAY_METHODS.has(name)) {
    return argumentMutation(node);
  }
  if (!OBJECT_METHODS.has(name) || !isGlobal(method.object, scope, "Object")) {
    return null;
  }
  return forwardedObjectTarget(node);
}

function forwardedObjectTarget(node) {
  const forward = methodName(unwrap(node.callee));
  if (forward === "call") {
    const target = node.arguments[1];
    if (!target || target.type === "SpreadElement") {
      return null;
    }
    return { targetNode: target, otherArgs: node.arguments.slice(2) };
  }
  const list = unwrap(node.arguments[1]);
  if (list?.type !== "ArrayExpression") {
    return null;
  }
  if (list.elements.some((element) => !element || element.type === "SpreadElement")) {
    return null;
  }
  const target = list.elements[0];
  if (!target) {
    return null;
  }
  return { targetNode: target, otherArgs: list.elements.slice(1) };
}

// `Object.assign.bind(thisArg)(target, ...sources)` mutates `target`.
// `Object.assign.bind(thisArg, target, ...bound)(...rest)` mutates that pre-bound target.
function boundObjectMutation(bound, node, scope) {
  const member = unwrap(bound.callee);
  if (!member || member.type !== "MemberExpression" || methodName(member) !== "bind") {
    return null;
  }
  const method = unwrap(member.object);
  if (!method || method.type !== "MemberExpression") {
    return null;
  }
  const name = methodName(method);
  if (!name || !OBJECT_METHODS.has(name) || !isGlobal(method.object, scope, "Object")) {
    return null;
  }
  if (bound.arguments.length > 1) {
    const target = bound.arguments[1];
    if (target.type === "SpreadElement") {
      return null;
    }
    return {
      targetNode: target,
      otherArgs: [...bound.arguments.slice(2), ...node.arguments],
    };
  }
  return argumentMutation(node);
}

function mutationCall(node, scope) {
  const callee = unwrap(node.callee);
  if (!callee) {
    return null;
  }
  if (callee.type === "CallExpression") {
    return boundObjectMutation(callee, node, scope);
  }
  if (callee.type !== "MemberExpression") {
    return null;
  }
  const name = methodName(callee);
  if (!name) {
    return null;
  }
  if (name === "call" || name === "apply") {
    return forwardedMutation(callee.object, node, scope);
  }
  if (ARRAY_METHODS.has(name)) {
    return { targetNode: callee.object, otherArgs: node.arguments };
  }
  if (REFLECT_METHODS.has(name) && isGlobal(callee.object, scope, "Reflect")) {
    return argumentMutation(node);
  }
  return objectMutation(node, scope, callee.object, name);
}

module.exports = { mutationCall };
