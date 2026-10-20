"use strict";

const api = require("./no-object-mutation-runtime");
const { constructInstance } = require("./no-object-mutation-class");
const { callFunction, escapeFunction } = require("./no-object-mutation-fn");
const { finishGet, visitKnownCall } = require("./no-object-mutation-invoke");
const { mutationCall } = require("./no-object-mutation-syntax");
const { readOnlyFresh } = require("./no-object-mutation-join");
const { callable, lookup, shared, unknown, unwrap } = require("./no-object-mutation-model");
const { considerWrite } = require("./no-object-mutation-write");

function visitArgument(state, argument, scope, role) {
  if (argument.type === "SpreadElement") {
    return api.visit(state, argument.argument, scope, role);
  }
  return api.visit(state, argument, scope, role);
}

function visitNew(state, node, scope, role) {
  const callee = unwrap(node.callee);
  let tag = unknown();
  if (
    callee?.type === "Identifier" &&
    (callee.name === "Map" || callee.name === "Set") &&
    !lookup(scope, callee.name)
  ) {
    tag = { type: "mapset" };
  } else {
    const classTag = api.visit(state, node.callee, scope, "read");
    tag = constructInstance(scope, classTag);
    for (const argument of node.arguments) {
      visitArgument(state, argument, scope, "escape");
    }
    runConstruction(state, classTag);
    if (role === "escape") {
      api.releaseEscaping(state, tag);
    }
    return tag;
  }
  for (const argument of node.arguments) {
    visitArgument(state, argument, scope, "escape");
  }
  if (role === "escape") {
    api.releaseEscaping(state, tag);
  }
  return tag;
}

function visitCall(state, node, scope, role) {
  const mutation = mutationCall(node, scope);
  if (mutation) {
    const target = api.visit(state, mutation.targetNode, scope, "read");
    const argRole = readOnlyFresh(target) ? "read" : "escape";
    const remembered = [];
    for (const argument of mutation.otherArgs) {
      remembered.push(visitArgument(state, argument, scope, argRole));
    }
    considerWrite(state, target, node, {
      viaCall: true,
      objectNode: mutation.targetNode,
      scope,
      remembered,
    });
    if (readOnlyFresh(target)) {
      return target;
    }
    return unknown();
  }
  const known = visitKnownCall(state, node, scope, role);
  if (known !== undefined) {
    return known;
  }
  const calleeTag = api.visit(state, node.callee, scope, "read");
  for (const argument of node.arguments) {
    visitArgument(state, argument, scope, "escape");
  }
  invokeCallable(state, calleeTag);
  return finishGet(node, calleeTag) || unknown();
}

function runConstruction(state, tag, seen = new Set()) {
  if (!tag || seen.has(tag)) {
    return;
  }
  seen.add(tag);
  if (tag.type === "union") {
    for (const member of tag.members) {
      runConstruction(state, member, seen);
    }
    return;
  }
  if (tag.type !== "fresh" || !tag.isClass) {
    return;
  }
  for (const item of tag.items) {
    if (item.methodKind === "constructor" || item.methodKind === "field") {
      callFunction(state, item);
    }
  }
}

function invokeCallable(state, tag) {
  if (tag.type === "union") {
    for (const member of tag.members) {
      invokeCallable(state, member);
    }
    return;
  }
  const fn = callable(tag);
  if (fn) {
    callFunction(state, fn);
  } else {
    return;
  }
}

function captureOuter(scope, tag, seen) {
  if (!tag || !scope.fnTag) {
    return;
  }
  const local = seen || new Set();
  if (local.has(tag)) {
    return;
  }
  local.add(tag);
  if (tag.type === "union") {
    for (const member of tag.members) {
      captureOuter(scope, member, local);
    }
    return;
  }
  if (tag.type === "fresh" && tag.ownerEnv !== scope.ownerEnv) {
    scope.fnTag.captured.push(tag);
  }
}

function visitIdentifier(state, node, scope, role) {
  let tag = lookup(scope, node.name)?.tag;
  // Unbound `arguments` is the function's parameter list, including through arrows.
  if (!tag && node.name === "arguments" && scope.hasArguments) {
    tag = shared("parameter");
  }
  if (role === "escape") {
    api.releaseEscaping(state, tag);
  }
  captureOuter(scope, tag);
  return tag || unknown();
}

function escapeExport(state, local, scope) {
  const fn = callable(lookup(scope, local.name)?.tag);
  if (fn) {
    escapeFunction(state, fn);
  } else {
    return;
  }
}

module.exports = {
  escapeExport,
  visitCall,
  visitIdentifier,
  visitNew,
};
