"use strict";

const seed = require("./no-object-mutation-seed");
const { TS_EXPRESSIONS, literalKey, methodName, unwrap } = require("./no-object-mutation-ast");

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
