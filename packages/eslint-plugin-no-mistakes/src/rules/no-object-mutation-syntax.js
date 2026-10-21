"use strict";

const seed = require("./no-object-mutation-seed");

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

module.exports = {
  ...seed,
  literalKey,
  memberKey,
  methodName,
  rootIdentifier,
  staticKey,
  staticPath,
  unwrap,
  TS_EXPRESSIONS,
};
