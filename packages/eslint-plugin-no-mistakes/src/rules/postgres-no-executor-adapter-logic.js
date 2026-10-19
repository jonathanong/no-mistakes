"use strict";

const { resolveVariable } = require("./postgres-query-text");
const { TS_ASSERTIONS } = require("./postgres-no-executor-adapter-binding");

function unwrapPlain(node) {
  let current = node;
  while (current && TS_ASSERTIONS.has(current.type)) current = current.expression;
  if (!current || current.type === "ChainExpression") return null;
  return current;
}

function unbound(node, context, name) {
  if (node?.type !== "Identifier" || node.name !== name) return false;
  const variable = resolveVariable(node, context);
  return !variable || variable.defs.length === 0;
}

function literalTruthiness(node, context) {
  const current = unwrapPlain(node);
  if (!current) return null;
  if (current.type === "UnaryExpression") {
    if (current.operator === "void") return false;
    if (current.operator !== "!") return null;
    const inner = literalTruthiness(current.argument, context);
    return inner == null ? null : !inner;
  }
  if (current.type === "Literal") {
    const { value } = current;
    if (value == null || value === false || value === "" || value === 0 || value === 0n)
      return false;
    return true;
  }
  if (current.type === "TemplateLiteral") {
    if (current.expressions.length > 0) return null;
    return current.quasis[0].value.cooked !== "";
  }
  if (unbound(current, context, "NaN") || unbound(current, context, "undefined")) return false;
  return null;
}

function isNullishLiteral(node, context) {
  const current = unwrapPlain(node);
  if (!current) return null;
  if (current.type === "Literal") return current.value == null;
  if (current.type === "UnaryExpression" && current.operator === "void") return true;
  if (unbound(current, context, "undefined")) return true;
  const truth = literalTruthiness(node, context);
  if (truth == null) return null;
  return false;
}

const MAYBE = new Set([
  "Identifier",
  "MemberExpression",
  "CallExpression",
  "ConditionalExpression",
  "LogicalExpression",
  "ChainExpression",
  "AwaitExpression",
  "YieldExpression",
  "TemplateLiteral",
]);

function asPossible(part) {
  if (!part?.definite) return part;
  return { ...part, definite: false };
}

// An operand whose truthiness is not a literal. `typeof` and comparisons are
// omitted so a value that cannot select the executor stays silent.
function isMaybe(node) {
  let current = node;
  while (current && TS_ASSERTIONS.has(current.type)) current = current.expression;
  if (current.type === "ChainExpression") return true;
  return MAYBE.has(current.type);
}

function propertyText(member) {
  if (!member.computed) {
    return member.property?.type === "Identifier" ? member.property.name : null;
  }
  const key = member.property;
  if (key?.type === "Literal" && typeof key.value === "string") return key.value;
  if (key?.type === "TemplateLiteral" && key.expressions.length === 0) {
    return key.quasis[0].value.cooked;
  }
  return null;
}

// `.bind` on an executor value still calls that executor. A callback receiver
// is not judged: bind can change this and prepend arguments.
function boundExecutor(call, resolve) {
  const callee = call.callee;
  if (callee.type !== "MemberExpression" || callee.optional) return null;
  if (propertyText(callee) !== "bind") return null;
  const object = resolve(callee.object);
  if (!object || object.adapter) return null;
  if (object.definite) {
    return {
      leaves: object.leaves,
      conditionalText: object.conditionalText ?? null,
      definite: true,
    };
  }
  return { leaves: object.leaves, conditionalText: null, definite: false };
}

function andPassed(node, resolve, context) {
  const truth = literalTruthiness(node.left, context);
  if (truth === false) return null;
  if (truth === true) return resolve(node.right);
  const left = resolve(node.left);
  if (left?.definite) return resolve(node.right);
  if (!isMaybe(node.left)) return null;
  return asPossible(resolve(node.right));
}

function orPassed(node, resolve, context, combine) {
  const truth = literalTruthiness(node.left, context);
  if (truth === true) return null;
  if (truth === false) return resolve(node.right);
  const left = resolve(node.left);
  if (left?.definite) return left;
  if (!isMaybe(node.left)) return null;
  return combine(left, resolve(node.right));
}

function nullishPassed(node, resolve, context, combine) {
  const left = resolve(node.left);
  if (left?.definite) return left;
  const nullish = isNullishLiteral(node.left, context);
  if (nullish === true) return resolve(node.right);
  if (nullish === false) return null;
  if (!isMaybe(node.left)) return null;
  return combine(left, resolve(node.right));
}

function logicalPassed(node, resolve, context, combine) {
  if (node.operator === "&&") return andPassed(node, resolve, context);
  if (node.operator === "||") return orPassed(node, resolve, context, combine);
  return nullishPassed(node, resolve, context, combine);
}

module.exports = {
  boundExecutor,
  logicalPassed,
};
