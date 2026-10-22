"use strict";

const api = require("./no-object-mutation-runtime");
const { readOnlyFresh } = require("./no-object-mutation-join");
const { unknown, unwrap } = require("./no-object-mutation-model");
const { considerWrite } = require("./no-object-mutation-write");

function visitArgument(state, argument, scope, role) {
  if (argument.type === "SpreadElement") {
    return api.visit(state, argument.argument, scope, role);
  }
  return api.visit(state, argument, scope, role);
}

function visitAliasedMutation(state, node, scope, method, forward) {
  let receiver = method.receiver;
  let receiverNode = method.receiverNode;
  let args = node.arguments;
  if (forward) {
    const thisNode = node.arguments[0];
    const list = forward === "apply" ? unwrap(node.arguments[1]) : null;
    if (
      !thisNode ||
      thisNode.type === "SpreadElement" ||
      (forward === "apply" &&
        (list?.type !== "ArrayExpression" ||
          list.elements.some((item) => !item || item.type === "SpreadElement")))
    ) {
      // Unsupported forwarding still evaluates its arguments. Keep writes and
      // escapes inside those expressions without guessing the outer receiver.
      for (const argument of node.arguments) visitArgument(state, argument, scope, "escape");
      return unknown();
    }
    const thisValue = api.visit(state, thisNode, scope, "read");
    if (!method.bound) {
      receiverNode = thisNode;
      receiver = thisValue;
    }
    if (forward === "apply") {
      args = list.elements;
      for (const argument of node.arguments.slice(2))
        visitArgument(state, argument, scope, "escape");
    } else {
      args = node.arguments.slice(1);
    }
  }
  const remembered = [...(method.boundArgs || [])];
  if (!readOnlyFresh(receiver)) {
    for (const value of remembered) api.releaseEscaping(state, value);
  }
  remembered.push(
    ...args.map((argument) =>
      visitArgument(state, argument, scope, readOnlyFresh(receiver) ? "read" : "escape"),
    ),
  );
  considerWrite(state, receiver, node, {
    viaCall: true,
    objectNode: receiverNode,
    scope,
    remembered,
  });
  return unknown();
}

module.exports = { visitAliasedMutation, visitArgument };
