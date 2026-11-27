"use strict";

const MAX_WALK = 32;
const IMPOSSIBLE = Symbol("impossible");
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

function numericText(node) {
  const value = unwrap(node);
  if (value?.type !== "Literal" || typeof value.value !== "number") return null;
  return String(value.value);
}

function literalKey(node) {
  const value = unwrap(node);
  if (value?.type === "UnaryExpression" && (value.operator === "-" || value.operator === "+")) {
    const inner = numericText(value.argument);
    if (inner == null) return null;
    return value.operator === "-" ? `-${inner}` : inner;
  }
  if (value?.type === "Literal") {
    if (typeof value.value === "bigint") return IMPOSSIBLE;
    if (
      typeof value.value === "string" ||
      typeof value.value === "number" ||
      typeof value.value === "boolean"
    ) {
      return String(value.value);
    }
    if (value.value === null) return "null";
    return null;
  }
  if (value?.type === "TemplateLiteral" && value.expressions.length === 0) {
    if (value.quasis.some((quasi) => quasi.value.cooked == null)) return null;
    return value.quasis.map((quasi) => quasi.value.cooked).join("");
  }
  return null;
}

function findVariable(scope, name) {
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

// One const hop. A later alias, let, var, parameter, or import stays unresolved.
function constSegment(node, context) {
  const value = unwrap(node);
  if (value?.type !== "Identifier" || !context.sourceCode?.getScope) return null;
  const variable = findVariable(context.sourceCode.getScope(value), value.name);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (!def || def.type !== "Variable" || def.parent?.kind !== "const") return null;
  if (def.node?.id?.type !== "Identifier" || !def.node.init) return null;
  return literalKey(def.node.init);
}

function staticProperty(member, resolveName) {
  if (member.computed) {
    const literal = literalKey(member.property);
    if (literal != null) return literal;
    const resolved = resolveName?.(member.property);
    return resolved == null ? null : resolved;
  }
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

function computeChain(member, resolveName) {
  const segments = [];
  let current = member;
  for (let steps = 0; current?.type === "MemberExpression"; steps += 1) {
    if (steps >= MAX_WALK) return null;
    segments.unshift(staticProperty(current, resolveName));
    current = unwrap(current.object);
  }
  return { segments, rooted: rooted(segments, current) };
}

function disposition(info, chain) {
  if (!info || info.segments.length < chain.parts.length) return null;
  const start = info.segments.length - chain.parts.length;
  const { rooted: fromRoot } = info;
  if (!chain.wildcard) {
    if (!fromRoot || start !== 0) return null;
  } else if (start === 0 && fromRoot) return null;
  let hole = false;
  for (let index = 0; index < chain.parts.length; index += 1) {
    const segment = info.segments[start + index];
    const part = chain.parts[index];
    if (segment == null) {
      if (part.startsWith("#")) return null;
      hole = true;
      continue;
    }
    if (segment !== part) return null;
  }
  return hole ? "unanalyzable" : "match";
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

function hasMatchingAncestor(node, chain, info, kind) {
  let current = node;
  for (;;) {
    const parent = surface(current).parent;
    if (!parent || parent.type !== "MemberExpression" || unwrap(parent.object) !== current) {
      return false;
    }
    const parentKind = disposition(info(parent), chain);
    if (parentKind === "match" || (kind !== "match" && parentKind === "unanalyzable")) return true;
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
  constSegment,
  computeChain,
  disposition,
  hasMatchingAncestor,
  invocationOf,
  expressionInfo,
  unwrap,
};
