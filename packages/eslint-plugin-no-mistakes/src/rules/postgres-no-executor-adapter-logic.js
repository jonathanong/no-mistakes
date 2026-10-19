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

// These values are always truthy, so `&&` always selects the right operand and
// `||` never does. `??` still treats them as non-nullish below.
const ALWAYS_TRUTHY = new Set([
  "ArrayExpression",
  "ArrowFunctionExpression",
  "ClassExpression",
  "FunctionExpression",
  "NewExpression",
  "ObjectExpression",
]);

function literalTruthiness(node, context) {
  const current = unwrapPlain(node);
  if (!current) return null;
  if (current.type === "UnaryExpression") {
    if (current.operator === "void") return false;
    if (current.operator === "typeof") return true;
    if (current.operator !== "!") return null;
    const inner = literalTruthiness(current.argument, context);
    return inner == null ? null : !inner;
  }
  if (ALWAYS_TRUTHY.has(current.type)) return true;
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

function asPossible(part) {
  if (!part?.definite) return part;
  return { ...part, definite: false };
}

// `??` follows only a left operand that can be nullish. Comparisons, `typeof`,
// `!`, unary `+`/`-`/`~`, `new`, and object, array, or function literals are
// never nullish, so `flag === 1 ?? read` stays silent.
const MAY_BE_NULLISH = new Set([
  "AssignmentExpression",
  "AwaitExpression",
  "CallExpression",
  "ChainExpression",
  "ConditionalExpression",
  "Identifier",
  "LogicalExpression",
  "MemberExpression",
  "SequenceExpression",
  "YieldExpression",
]);

function mayBeNullish(node) {
  let current = node;
  while (current && TS_ASSERTIONS.has(current.type)) current = current.expression;
  return MAY_BE_NULLISH.has(current.type);
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

// `.bind` on an executor value still calls that executor. Arguments after
// `this` are prepended, so `read.bind(null, account)` is not `read(sql`…`)`.
// A callback receiver is not judged: bind can change `this`.
function boundExecutor(call, resolve) {
  const callee = call.callee;
  if (callee.type !== "MemberExpression" || callee.optional) return null;
  if (propertyText(callee) !== "bind") return null;
  if (call.arguments.length > 1) return null;
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

// An unknown left operand can be truthy or falsy, so `&&` / `||` can select
// the executor. Only proven executor leaves are named.
function andPassed(node, resolve, context) {
  const truth = literalTruthiness(node.left, context);
  if (truth === false) return null;
  if (truth === true) return resolve(node.right);
  const left = resolve(node.left);
  if (left?.definite) return resolve(node.right);
  return asPossible(resolve(node.right));
}

function orPassed(node, resolve, context, combine) {
  const truth = literalTruthiness(node.left, context);
  if (truth === true) return null;
  if (truth === false) return resolve(node.right);
  const left = resolve(node.left);
  if (left?.definite) return left;
  return combine(left, resolve(node.right));
}

function nullishPassed(node, resolve, context, combine) {
  const left = resolve(node.left);
  if (left?.definite) return left;
  const nullish = isNullishLiteral(node.left, context);
  if (nullish === true) return resolve(node.right);
  if (nullish === false) return null;
  if (!mayBeNullish(node.left)) return null;
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
