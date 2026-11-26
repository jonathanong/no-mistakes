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

// null means the argument's position is opaque. undefined means it is absent.
function argumentAt(args, index) {
  if (!args) return null;
  for (let position = 0; position <= index && position < args.length; position += 1) {
    if (args[position]?.type === "SpreadElement") return null;
  }
  return args[index] ?? undefined;
}

// Target-aware callers can stop at configured call/apply methods. Unmatched
// intrinsic chains use the explicit receiver from the outer call.
function logicalCall(node, stopAt, preserveOpaqueArguments = false) {
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
    if (!preserveOpaqueArguments && args.some((arg) => arg?.type === "SpreadElement")) {
      return null;
    }
    callee = stopAt && forwarded ? thisArg : callee.object;
    thisArg = argumentAt(args, 0);
    forwarded = true;
    if (name === "call") {
      args = thisArg === null ? null : args.slice(1);
      continue;
    }
    const list = unwrapExpression(argumentAt(args, 1));
    if (list?.type === "ArrayExpression") {
      if (
        !preserveOpaqueArguments &&
        list.elements.some((element) => !element || element.type === "SpreadElement")
      ) {
        return null;
      }
      args = list.elements;
    } else if (preserveOpaqueArguments) {
      args = list === undefined || (list?.type === "Literal" && list.value === null) ? [] : null;
    } else {
      return null;
    }
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

module.exports = { argumentAt, propertyName, memberPropertyName, logicalCall, literalString };
