"use strict";

const { ruleSuppression } = require("../rule-suppression");

const MAX_WALK = 32;
const MAX_SEGMENTS = 16;
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

function compile(entry) {
  const parts = Array.isArray(entry.path) ? entry.path.slice() : entry.path.split(".");
  if (parts.length > MAX_SEGMENTS || parts.some((part) => part.length === 0)) return null;
  let wildcard = false;
  let body = parts;
  if (body[0] === "*") {
    wildcard = true;
    body = body.slice(1);
  }
  if (entry.receiver === "*") wildcard = true;
  else if (typeof entry.receiver === "string") {
    body = [entry.receiver, ...body];
    wildcard = false;
  }
  if (body.length === 0 || body.length > MAX_SEGMENTS || body.some((part) => part === "*"))
    return null;
  return { wildcard, parts: body, message: entry.message };
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
    if (!parent || parent.type !== "MemberExpression" || unwrap(parent.object) !== current)
      return false;
    if (matches(info(parent), chain)) return true;
    current = parent;
  }
}

function invocationOf(member) {
  const parent = surface(member).parent;
  if (parent.type !== "CallExpression" && parent.type !== "NewExpression") return null;
  return unwrap(parent.callee) === member ? parent : null;
}

const segment = { type: "string", minLength: 1, maxLength: 128 };
module.exports = {
  meta: {
    type: "problem",
    docs: { description: "ban configured static member-access chains", recommended: false },
    schema: [
      {
        type: "object",
        additionalProperties: false,
        properties: {
          chains: {
            type: "array",
            maxItems: 64,
            items: {
              type: "object",
              additionalProperties: false,
              required: ["path", "message"],
              properties: {
                path: {
                  anyOf: [
                    { type: "string", minLength: 1, maxLength: 256 },
                    { type: "array", minItems: 1, maxItems: MAX_SEGMENTS, items: segment },
                  ],
                },
                receiver: segment,
                message: { type: "string", minLength: 1, maxLength: 512 },
              },
            },
          },
        },
      },
    ],
    messages: { forbiddenChain: "{{message}}" },
  },
  create(context) {
    const compiled = [];
    const entries = context.options[0]?.chains;
    if (Array.isArray(entries)) {
      for (const entry of entries) {
        const chain = compile(entry);
        if (chain) compiled.push(chain);
      }
    }
    const suppressed = ruleSuppression(context, "forbidden-member-chains");
    const cache = new WeakMap();
    const info = (member) => {
      if (cache.has(member)) return cache.get(member);
      const value = computeChain(member);
      cache.set(member, value);
      return value;
    };
    return {
      MemberExpression(node) {
        if (compiled.length === 0) return;
        for (const chain of compiled) {
          if (!matches(info(node), chain) || hasMatchingAncestor(node, chain, info)) continue;
          const target = invocationOf(node) || node;
          if (suppressed(target)) continue;
          context.report({
            node: target,
            messageId: "forbiddenChain",
            data: { message: chain.message },
          });
        }
      },
    };
  },
};
