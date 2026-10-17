"use strict";

const MAX_WALK = 32;
const TRANSPARENT = new Set([
  "ChainExpression",
  "ParenthesizedExpression",
  "TSAsExpression",
  "TSSatisfiesExpression",
  "TSNonNullExpression",
  "TSTypeAssertion",
  "TSInstantiationExpression",
  "TypeCastExpression",
]);

function unwrap(node) {
  let current = node;
  while (current && TRANSPARENT.has(current.type)) current = current.expression;
  return current;
}

function literalKey(node) {
  const value = unwrap(node);
  if (
    value?.type === "Literal" &&
    (typeof value.value === "string" || typeof value.value === "number")
  ) {
    return String(value.value);
  }
  if (value?.type === "TemplateLiteral" && value.expressions.length === 0) {
    return value.quasis.map((quasi) => quasi.value.cooked).join("");
  }
  return null;
}

function staticProperty(member) {
  if (member.computed) return literalKey(member.property);
  const property = member.property;
  return property.type === "PrivateIdentifier" ? `#${property.name}` : property.name;
}

function rooted(segments, node) {
  if (node?.type === "Identifier") segments.unshift(node.name);
  else if (node?.type === "ThisExpression") segments.unshift("this");
  else if (node?.type === "Super") segments.unshift("super");
  else if (node?.type === "MetaProperty") segments.unshift(node.meta.name, node.property.name);
  else return false;
  return true;
}

function computeChain(member) {
  const segments = [];
  let current = member;
  for (let steps = 0; current?.type === "MemberExpression"; steps += 1) {
    if (steps >= MAX_WALK) return null;
    const name = staticProperty(current);
    if (name == null) return { segments, rooted: false };
    segments.unshift(name);
    current = unwrap(current.object);
  }
  return { segments, rooted: rooted(segments, current) };
}

function matches(info, chain) {
  if (!info) return false;
  const { segments, rooted: fromRoot } = info;
  const { parts } = chain;
  if (segments.length < parts.length) return false;
  const start = segments.length - parts.length;
  for (let index = 0; index < parts.length; index += 1) {
    if (segments[start + index] !== parts[index]) return false;
  }
  // A wildcard needs a receiver. An opaque receiver is not one of the segments.
  if (!chain.wildcard) return fromRoot && start === 0;
  return start > 0 || !fromRoot;
}

function surface(node) {
  let current = node;
  while (
    current.parent &&
    TRANSPARENT.has(current.parent.type) &&
    unwrap(current.parent) === node
  ) {
    current = current.parent;
  }
  return current;
}

function hasMatchingAncestor(node, chain, info) {
  let current = node;
  for (;;) {
    const parent = surface(current).parent;
    if (!parent || parent.type !== "MemberExpression" || unwrap(parent.object) !== current) {
      return false;
    }
    if (matches(info(parent), chain)) return true;
    current = parent;
  }
}

function invocationOf(member) {
  const parent = surface(member).parent;
  if (parent.type !== "CallExpression" && parent.type !== "NewExpression") return null;
  return unwrap(parent.callee) === member ? parent : null;
}

function expressionInfo(node, memberInfo) {
  const value = unwrap(node);
  if (value.type === "MemberExpression") return memberInfo(value);
  const segments = [];
  return { segments, rooted: rooted(segments, value) };
}

module.exports = {
  matches,
  computeChain,
  hasMatchingAncestor,
  invocationOf,
  expressionInfo,
};
