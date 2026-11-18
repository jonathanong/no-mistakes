"use strict";

const api = require("./no-object-mutation-runtime");
const { checkedArray } = require("./no-object-mutation-array-proof");
const { unsafeReceiver } = require("./no-object-mutation-iteration-aliases");
const { callFunction } = require("./no-object-mutation-fn");
const { walkLoop } = require("./no-object-mutation-loop");
const { lookup, readMember, unknown } = require("./no-object-mutation-values");
const { mutationCall } = require("./no-object-mutation-forward");
const { methodName, rootIdentifier, unwrap } = require("./no-object-mutation-syntax");

const ITERATIONS = new Set(["map", "forEach", "flatMap", "filter"]);

function visitIteration(state, node, scope) {
  const callee = unwrap(node.callee);
  if (callee?.type !== "MemberExpression" || !ITERATIONS.has(methodName(callee))) return;
  if (![1, 2].includes(node.arguments.length)) return;
  const callback = unwrap(node.arguments[0]);
  if (!["ArrowFunctionExpression", "FunctionExpression"].includes(callback?.type)) return;
  if (callback.async || callback.generator || node.arguments[1]?.type === "SpreadElement") return;
  const object = unwrap(callee.object);
  const cell = object.type === "Identifier" ? lookup(scope, object.name) : null;
  const literal = object.type === "ArrayExpression";
  if (unsupportedWrites(callback.body, scope)) return;
  const receiver = api.visit(state, callee.object, scope, "read");
  const name = methodName(callee);
  const proven =
    literal ||
    cell?.arrayParameter ||
    receiver.arrayProducer ||
    (receiver.type === "fresh" && receiver.isArray);
  const legacy = literal || cell?.arrayParameter || (cell?.tag.isArray && !receiver.arrayProducer);
  const checked = legacy ? null : checkedArray(state, object, scope, name, Boolean(proven));
  if (
    checked === false ||
    (!proven && checked !== true) ||
    state.arrayPrototypeChanged ||
    (!legacy && state.builtins.Array.iterationChanged) ||
    unsafeReceiver(state, receiver, !legacy)
  ) {
    return { fallbackCalleeTag: readMember(receiver, name) };
  }
  const tag = api.visit(state, callback, scope, "read");
  if (node.arguments[1]) tag.receiver = api.visit(state, node.arguments[1], scope, "read");
  walkLoop(
    state,
    () => {
      tag.walked = false;
      callFunction(state, tag);
    },
    scope,
  );
  return unknown();
}

function unsupportedWrites(node, scope) {
  if (!node || typeof node.type !== "string") return false;
  if (
    (node.type === "AssignmentExpression" && unwrap(node.left).type === "MemberExpression") ||
    (node.type === "UpdateExpression" && unwrap(node.argument).type === "MemberExpression")
  )
    return true;
  if (node.type === "CallExpression") {
    const mutation = mutationCall(node, scope);
    if (mutation && !mutation.arrayReceiver) return true;
  }
  if (node.type === "MemberExpression") {
    const captured = lookup(scope, rootIdentifier(node));
    if (captured?.tag.type === "fresh" || unwrap(node.object).type === "ThisExpression") {
      const call = node.parent;
      const mutation = call?.type === "CallExpression" && mutationCall(call, scope);
      if (
        !mutation?.arrayReceiver ||
        !["Identifier", "ThisExpression"].includes(unwrap(mutation.targetNode).type)
      )
        return true;
    }
  }
  return Object.entries(node).some(([key, value]) => {
    if (["parent", "loc", "range", "tokens"].includes(key)) return false;
    return Array.isArray(value)
      ? value.some((part) => unsupportedWrites(part, scope))
      : unsupportedWrites(value, scope);
  });
}

module.exports = { visitIteration };
