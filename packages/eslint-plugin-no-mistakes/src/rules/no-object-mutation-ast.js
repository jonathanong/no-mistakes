"use strict";

// Leaf shared by syntax and forward. Those two files must not require each
// other: a cycle makes `mutationCall` undefined when forward loads first.

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

function methodName(member) {
  if (member.property.type === "Identifier" && !member.computed) {
    return member.property.name;
  }
  return member.computed ? literalKey(member.property) : null;
}

module.exports = {
  TS_EXPRESSIONS,
  literalKey,
  methodName,
  unwrap,
};
