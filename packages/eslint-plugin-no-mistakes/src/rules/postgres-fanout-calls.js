"use strict";

const { memberPropertyName } = require("./postgres-executor");
const { resolveVariable, unwrapTs } = require("./postgres-query-text");

function staticMethod(node) {
  node = unwrapTs(node);
  if (node?.type !== "MemberExpression") return null;
  const key = unwrapTs(node.property);
  if (
    node.computed &&
    key.type !== "Literal" &&
    !(key.type === "TemplateLiteral" && key.expressions.length === 0)
  )
    return null;
  return { node, name: memberPropertyName(node) };
}

// Recover the actual map receiver: xs.map.call(ys, fn) maps ys, not xs.
function forwardedMethod(expression, context, seen = new Set()) {
  const node = unwrapTs(expression);
  if (node?.type === "SequenceExpression")
    return forwardedMethod(node.expressions.at(-1), context, seen);
  const direct = staticMethod(node);
  if (direct) return direct;
  if (node?.type !== "Identifier") return null;
  const variable = resolveVariable(node, context);
  if (!variable || seen.has(variable) || variable.defs.length !== 1) return null;
  seen.add(variable);
  if (variable.references.some((ref) => ref.isWrite() && !ref.init)) return null;
  const def = variable.defs[0];
  if (def.parent?.kind !== "const" || def.node.id.type !== "Identifier") return null;
  return forwardedMethod(def.node.init, context, seen);
}

function invokedMethod(call, context) {
  call = unwrapTs(call);
  if (call?.type !== "CallExpression") return null;
  let method = staticMethod(call.callee);
  let args = call.arguments;
  let receiver;
  if (method?.name === "call" || method?.name === "apply") {
    if (args.some((arg) => arg.type === "SpreadElement")) return null;
    receiver = unwrapTs(args[0]);
    if (method.name === "call") args = args.slice(1);
    else {
      const array = unwrapTs(args[1]);
      if (
        array?.type !== "ArrayExpression" ||
        array.elements.some((arg) => !arg || arg.type === "SpreadElement")
      )
        return null;
      args = array.elements;
    }
    method = forwardedMethod(method.node.object, context);
  }
  if (!method) return null;
  return { callee: method.node, name: method.name, arguments: args, receiver };
}

function fanoutMapCall(call, isPromiseAll, context) {
  const outer = invokedMethod(call, context);
  if (!outer || !isPromiseAll(outer.callee)) return null;
  const mapped = invokedMethod(outer.arguments[0], context);
  if (mapped?.name !== "map") return null;
  return {
    source: mapped.receiver ?? unwrapTs(mapped.callee.object),
    callback: mapped.arguments[0],
  };
}

module.exports = { fanoutMapCall };
