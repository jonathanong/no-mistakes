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
  if (member.computed) {
    return literalKey(member.property);
  }
  return null;
}

function isGlobalObject(node, scope) {
  const target = unwrap(node);
  return Boolean(
    target?.type === "Identifier" && target.name === "Object" && !lookup(scope, "Object"),
  );
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
  if (ARRAY_METHODS.has(name)) {
    return { targetNode: callee.object, otherArgs: node.arguments };
  }
  if (!OBJECT_METHODS.has(name) || !isGlobalObject(callee.object, scope)) {
    return null;
  }
  const target = node.arguments[0];
  if (!target || target.type === "SpreadElement") {
    return null;
  }
  return { targetNode: target, otherArgs: node.arguments.slice(1) };
}

module.exports = {
  ...seed,
  literalKey,
  memberKey,
  mutationCall,
  rootIdentifier,
  staticKey,
  staticPath,
  unwrap,
  TS_EXPRESSIONS,
};
