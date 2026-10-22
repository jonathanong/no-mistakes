"use strict";

const api = require("./no-object-mutation-runtime");
const { iteratedContents } = require("./no-object-mutation-read");
const { methodName, unwrap } = require("./no-object-mutation-syntax");
const {
  callable,
  createFresh,
  joinValues,
  lookup,
  remember,
  unknown,
} = require("./no-object-mutation-values");

function memberCall(node, name) {
  const callee = unwrap(node.callee);
  if (!callee || callee.type !== "MemberExpression" || methodName(callee) !== name) {
    return null;
  }
  return callee;
}

function visitKnownCall(state, node, scope, role) {
  const values = memberCall(node, "values");
  if (values && isGlobalObject(values.object, scope)) {
    const target = node.arguments[0];
    if (target && target.type !== "SpreadElement") {
      const tag = api.visit(state, target, scope, "read");
      for (const argument of node.arguments.slice(1)) {
        api.visit(
          state,
          argument.type === "SpreadElement" ? argument.argument : argument,
          scope,
          "read",
        );
      }
      return arrayOf(scope, iteratedContents(tag));
    }
  }
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
    if (role === "escape") api.releaseEscaping(state, tag);
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
  if (role === "escape") {
    api.releaseEscaping(state, tag);
  }
  return tag;
}

function isGlobalObject(node, scope) {
  const target = unwrap(node);
  return Boolean(
    target?.type === "Identifier" && target.name === "Object" && !lookup(scope, "Object"),
  );
}

function arrayOf(scope, tag) {
  const fresh = createFresh(scope);
  remember(fresh, tag);
  return fresh;
}

function boundTag(tag) {
  if (tag.type === "unknown") {
    return unknown();
  }
  if (tag.type === "union") {
    let joined = unknown();
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
  if (!memberCall(node, "get")) {
    return null;
  }
  return getContents(calleeTag);
}

module.exports = { finishGet, visitKnownCall };
