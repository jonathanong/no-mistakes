"use strict";
const { unwrapExpression } = require("./async-ast");

function propertyName(node) {
  if (!node) return null;
  if (node.type === "Literal") return String(node.value);
  if (typeof node.name === "string") return node.name;
  return literalString(node);
}

function memberPropertyName(node) {
  let current = node;
  while (current?.type === "ChainExpression") current = current.expression;
  if (current?.type !== "MemberExpression") return null;
  if (current.computed && current.property?.type === "Identifier") return null;
  return propertyName(current.property);
}

// The call that actually runs, after optional chaining, a type or non-null
// assertion, a comma callee, and `.call` / a literal `.apply` array. Opaque
// `.apply` or a spread `.call` returns null so callers do not read the wrong
// argument. A target-aware caller can stop at configured call/apply methods;
// unmatched intrinsic chains then use the explicit receiver from the outer call.
function logicalCall(node, stopAt) {
  if (node?.type !== "CallExpression") return null;
  let callee = unwrapExpression(node.callee);
  let args = node.arguments;
  let forwarded = false;
  let thisArg;
  for (let depth = 0; depth < 4; depth += 1) {
    callee = unwrapExpression(callee);
    if (stopAt?.(callee)) break;
    if (callee?.type === "SequenceExpression" && callee.expressions.length > 0) {
      callee = callee.expressions[callee.expressions.length - 1];
      if (!forwarded) thisArg = null;
      forwarded = true;
      continue;
    }
    if (callee?.type !== "MemberExpression") break;
    const name = memberPropertyName(callee);
    if (name !== "call" && name !== "apply") break;
    if (name === "call") {
      if (args.some((arg) => arg?.type === "SpreadElement")) return null;
      callee = stopAt && forwarded ? thisArg : callee.object;
      thisArg = args[0];
      forwarded = true;
      args = args.slice(1);
      continue;
    }
    if (args.some((arg) => arg?.type === "SpreadElement")) return null;
    const list = unwrapExpression(args[1]);
    if (
      list?.type !== "ArrayExpression" ||
      list.elements.some((element) => !element || element.type === "SpreadElement")
    ) {
      return null;
    }
    callee = stopAt && forwarded ? thisArg : callee.object;
    thisArg = args[0];
    forwarded = true;
    args = list.elements;
  }
  return { callee: unwrapExpression(callee), arguments: args, forwarded };
}

function literalString(node) {
  if (!node) return null;
  if (node.type === "Literal" && typeof node.value === "string") return node.value;
  if (node.type === "TemplateLiteral" && node.expressions.length === 0) {
    return node.quasis.map((quasi) => quasi.value.cooked ?? quasi.value.raw).join("");
  }
  return null;
}

module.exports = { propertyName, memberPropertyName, logicalCall, literalString };
