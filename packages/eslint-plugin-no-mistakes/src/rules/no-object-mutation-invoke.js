"use strict";

const api = require("./no-object-mutation-runtime");
const { escapeResult } = require("./no-object-mutation-result");
const { callFunction } = require("./no-object-mutation-fn");
const { visitIteration } = require("./no-object-mutation-iteration");
const { reducerCallback } = require("./no-object-mutation-reduce");
const { arrayFactory } = require("./no-object-mutation-array-factory");
const { methodName, staticKey, unwrap } = require("./no-object-mutation-syntax");
const { callable, joinValues, readMember, unknown } = require("./no-object-mutation-values");

function memberCall(node, name) {
  const callee = unwrap(node.callee);
  if (!callee || callee.type !== "MemberExpression" || methodName(callee) !== name) {
    return null;
  }
  return callee;
}

function visitKnownCall(state, node, scope, role) {
  const iteration = visitIteration(state, node, scope, role);
  if (iteration !== undefined) return iteration;
  const reducer = reducerCallback(node);
  if (reducer) {
    const callee = unwrap(node.callee);
    const receiver = api.visit(state, callee.object, scope, "read");
    const method = readMember(receiver, methodName(callee));
    if (
      callable(method) ||
      (receiver.type === "fresh" &&
        (receiver.fields.has(methodName(callee)) ||
          receiver.items.some(
            (item) =>
              item.node?.parent?.key && staticKey(item.node.parent.key) === methodName(callee),
          )))
    ) {
      // Reuse the evaluated receiver in the ordinary call path. A custom
      // method may supply a shared accumulator or retain its callback.
      return { fallbackCalleeTag: method };
    }
    const seed = api.visit(state, node.arguments[1], scope, "read");
    const callback = api.visit(state, reducer, scope, "read");
    callback.reduceSeed = seed;
    const escapeEpoch = state.escapeEpoch;
    callFunction(state, callback);
    if (state.escapeEpoch !== escapeEpoch) {
      // A leak at the end of one iteration is visible at the next entry.
      callback.walked = false;
      callFunction(state, callback);
    }
    seed.ownerEnv = scope.ownerEnv;
    escapeResult(state, seed, scope, role);
    return seed;
  }
  const array = arrayFactory(state, node, scope);
  if (array !== undefined) return array;
  const bound = memberCall(node, "bind");
  if (!bound) {
    return undefined;
  }
  const source = api.visit(state, bound.object, scope, "read");
  if (source.mutator) {
    const thisNode = node.arguments[0];
    const thisValue = thisNode
      ? api.visit(
          state,
          thisNode.type === "SpreadElement" ? thisNode.argument : thisNode,
          scope,
          "read",
        )
      : unknown();
    const boundArgs = [...(source.mutator.boundArgs || [])];
    for (const argument of node.arguments.slice(1)) {
      boundArgs.push(
        api.visit(
          state,
          argument.type === "SpreadElement" ? argument.argument : argument,
          scope,
          "read",
        ),
      );
    }
    const tag = {
      type: "unknown",
      mutator: {
        receiver: source.mutator.bound
          ? source.mutator.receiver
          : thisNode?.type === "SpreadElement"
            ? unknown()
            : thisValue,
        receiverNode: source.mutator.bound ? source.mutator.receiverNode : thisNode,
        bound: true,
        boundArgs,
        member: unknown(),
      },
    };
    escapeResult(state, tag, scope, role);
    return tag;
  }
  const tag = boundTag(source);
  for (const argument of node.arguments) {
    api.visit(
      state,
      argument.type === "SpreadElement" ? argument.argument : argument,
      scope,
      "escape",
    );
  }
  escapeResult(state, tag, scope, role);
  return tag;
}

function boundTag(tag) {
  if (tag.type === "unknown") {
    return unknown();
  }
  if (tag.type === "union") {
    let joined = unknown(true);
    for (const member of tag.members) {
      joined = joinValues(joined, boundTag(member));
    }
    return joined;
  }
  return callable(tag) || unknown();
}

function getContents(tag) {
  if (tag.type === "unknown" || tag.type === "fn") {
    return null;
  }
  if (tag.type === "union") {
    let joined = null;
    for (const member of tag.members) {
      const part = getContents(member);
      if (part) {
        joined = joined ? joinValues(joined, part) : part;
      }
    }
    return joined;
  }
  if (tag.type === "shared" || tag.type === "this") {
    return tag;
  }
  if (tag.type === "mapset") {
    return tag.contents || unknown();
  }
  return null;
}

function finishGet(node, calleeTag) {
  // Iterating a caller-owned collection yields its caller-owned elements.
  const iterator = ["entries", "values"].find((name) => memberCall(node, name));
  if (iterator && calleeTag.type === "shared" && !node.arguments.length) return calleeTag;
  if (!memberCall(node, "get")) {
    return null;
  }
  return getContents(calleeTag);
}

module.exports = { finishGet, visitKnownCall };
