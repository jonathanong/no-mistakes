"use strict";

const { lookup } = require("./no-object-mutation-values");
const seed = require("./no-object-mutation-seed");

const ARRAY_METHODS = new Set([
  "copyWithin",
  "fill",
  "pop",
  "push",
  "reverse",
  "shift",
  "sort",
  "splice",
  "unshift",
]);
const OBJECT_METHODS = new Set(["assign", "defineProperties", "defineProperty", "setPrototypeOf"]);
const REFLECT_METHODS = new Set(["defineProperty", "set"]);
const TS_EXPRESSIONS = new Set([
  "TSAsExpression",
  "TSNonNullExpression",
  "TSSatisfiesExpression",
  "TSTypeAssertion",
]);

function unwrap(node) {
  let current = node;
  while (
    current &&
    (current.type === "ChainExpression" ||
      current.type === "ParenthesizedExpression" ||
      TS_EXPRESSIONS.has(current.type))
  ) {
    current = current.expression;
  }
  return current;
}

function literalKey(node) {
  const target = unwrap(node);
  if (
    target.type === "Literal" &&
    (typeof target.value === "string" || typeof target.value === "number")
  ) {
    return String(target.value);
  }
  if (target.type === "TemplateLiteral" && target.expressions.length === 0) {
    return target.quasis.map((quasi) => quasi.value.cooked).join("");
  }
  return null;
}

function staticKey(node) {
  const target = unwrap(node);
  if (target.type === "Identifier") {
    return target.name;
  }
  return literalKey(target);
}

function memberKey(member) {
  if (member.type !== "MemberExpression") {
    return null;
  }
  if (member.computed) {
    return literalKey(member.property);
  }
  if (member.property.type === "Identifier") {
    return member.property.name;
  }
  return null;
}

function staticPath(node) {
  const target = unwrap(node);
  if (target.type === "Identifier") {
    return target.name;
  }
  if (target.type !== "MemberExpression") {
    return null;
  }
  const objectPath = staticPath(target.object);
  const key = memberKey(target);
  if (objectPath && key) {
    return `${objectPath}.${key}`;
  }
  return null;
}

function rootIdentifier(node) {
  let current = unwrap(node);
  while (current?.type === "MemberExpression") {
    current = unwrap(current.object);
  }
  if (current?.type === "Identifier") {
    return current.name;
  }
  return null;
}

function methodName(member) {
  if (member.property.type === "Identifier" && !member.computed) {
    return member.property.name;
  }
  return member.computed ? literalKey(member.property) : null;
}

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
  } else {
    return null;
  }
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

function mutationCall(node, scope) {
  const callee = unwrap(node.callee);
  if (!callee || callee.type !== "MemberExpression") {
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

module.exports = {
  ...seed,
  literalKey,
  memberKey,
  methodName,
  mutationCall,
  rootIdentifier,
  staticKey,
  staticPath,
  unwrap,
  TS_EXPRESSIONS,
};
