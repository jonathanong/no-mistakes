"use strict";

const api = require("./no-object-mutation-runtime");
const { bindName, bindPattern } = require("./no-object-mutation-bind");
const {
  functionStatements,
  hoistedFn,
  seedClasses,
  seedHoisted,
} = require("./no-object-mutation-syntax");
const { child, lookup, shared } = require("./no-object-mutation-values");
const { markEscaped, report } = require("./no-object-mutation-write");

function registerFn(tag) {
  tag.registered = true;
  tag.scope.functionEnv.pending.push(tag);
}

function flushPending(state, scope) {
  const pending = scope.pending;
  scope.pending = [];
  for (const tag of pending) {
    if (!tag.walked && tag.node) {
      walkFunctionBody(state, tag.node, tag.scope, tag);
    }
  }
}

function ensureWalked(state, tag) {
  if (tag.walked) {
    return;
  }
  walkFunctionBody(state, tag.node, tag.scope, tag);
}

function visitFunction(state, node, scope, role) {
  const existing = node.type === "FunctionDeclaration" ? hoistedFn(node, scope) : null;
  const tag = existing || {
    type: "fn",
    deferred: [],
    captured: [],
    didEscape: false,
    walked: false,
    registered: false,
    node,
    scope,
  };
  if (!existing) {
    registerFn(tag);
    if (node.type === "FunctionDeclaration" && node.id) {
      bindName(node.id.name, tag, scope, "local");
    }
  }
  if (role === "escape") {
    escapeFunction(state, tag);
  }
  if (!existing) {
    return tag;
  }
  return lookup(scope, node.id.name)?.tag || tag;
}

function walkFunctionBody(state, node, scope, tag) {
  tag.walked = true;
  const fnEnv = child(scope);
  fnEnv.ownerEnv = fnEnv;
  fnEnv.functionEnv = fnEnv;
  fnEnv.fnTag = tag;
  fnEnv.pending = [];
  // Arrows keep the enclosing `this`. A nested function has its own.
  const arrow = node.type === "ArrowFunctionExpression";
  const classMethod =
    !arrow &&
    (node.parent?.type === "MethodDefinition" ||
      node.parent?.type === "TSAbstractMethodDefinition");
  fnEnv.inClass = arrow ? Boolean(scope.inClass) : classMethod;
  fnEnv.hasArguments = arrow ? Boolean(scope.hasArguments) : true;
  fnEnv.receiver = arrow ? scope.receiver || null : tag.receiver || null;
  if (tag.reduceSeed) tag.reduceSeed.ownerEnv = fnEnv;
  for (const [index, param] of node.params.entries()) {
    if (index === 0 && tag.reduceSeed) {
      bindName(param.name, tag.reduceSeed, fnEnv, "param");
    } else {
      bindPattern(state, param, shared("parameter"), fnEnv, "param");
    }
  }
  const body = functionStatements(node);
  if (Array.isArray(body)) {
    seedHoisted(body, fnEnv, false, registerFn);
    seedClasses(body, fnEnv);
    api.visitStatements(state, body, fnEnv);
  } else {
    api.visit(state, body, fnEnv, "escape");
  }
  flushPending(state, fnEnv);
}

function deferExpression(state, expression, scope) {
  const tag = {
    type: "fn",
    deferred: [],
    captured: [],
    didEscape: false,
    walked: true,
    registered: true,
    node: expression,
    scope,
    methodKind: "field",
  };
  const fnEnv = child(scope);
  fnEnv.ownerEnv = fnEnv;
  fnEnv.functionEnv = fnEnv;
  fnEnv.fnTag = tag;
  fnEnv.inClass = Boolean(scope.inClass);
  fnEnv.pending = [];
  fnEnv.hasArguments = Boolean(scope.hasArguments);
  // A field's `this` is the instance, not an enclosing method receiver.
  fnEnv.receiver = null;
  api.visit(state, expression, fnEnv, "read");
  flushPending(state, fnEnv);
  return tag;
}

function callFunction(state, tag) {
  ensureWalked(state, tag);
  for (const write of tag.deferred) {
    if (write.fresh?.type === "fresh" && write.fresh.escaped) {
      report(state, write.node, write.objectNode, write.viaCall);
    }
  }
}

function escapeFunction(state, tag) {
  if (!tag || tag.didEscape) {
    return;
  }
  tag.didEscape = true;
  ensureWalked(state, tag);
  for (const fresh of tag.captured) {
    markEscaped(state, fresh);
  }
  for (const write of tag.deferred) {
    report(state, write.node, write.objectNode, write.viaCall);
  }
}

function escapeNestedFunctions(state, tag, seen = new Set()) {
  if (!tag || seen.has(tag)) {
    return;
  }
  seen.add(tag);
  if (tag.mutator) {
    escapeNestedFunctions(state, tag.mutator.receiver, seen);
    for (const value of tag.mutator.boundArgs || []) escapeNestedFunctions(state, value, seen);
  }
  if (tag.type === "union") {
    for (const member of tag.members) {
      escapeNestedFunctions(state, member, seen);
    }
    return;
  }
  if (tag.type === "fn") {
    escapeFunction(state, tag);
    return;
  }
  if (tag.type !== "fresh") {
    return;
  }
  for (const field of tag.fields.values()) {
    escapeNestedFunctions(state, field, seen);
  }
  for (const item of tag.items) {
    escapeNestedFunctions(state, item, seen);
  }
  escapeNestedFunctions(state, tag.dynamic, seen);
}

function releaseEscaping(state, tag) {
  if (tag == null) {
    return;
  }
  if (tag.mutator) {
    releaseEscaping(state, tag.mutator.receiver);
    for (const value of tag.mutator.boundArgs || []) releaseEscaping(state, value);
  }
  if (tag.type === "union") {
    for (const member of tag.members) {
      releaseEscaping(state, member);
    }
    return;
  }
  if (tag.type === "fn") {
    escapeFunction(state, tag);
    return;
  }
  if (tag.fn?.type === "fn") {
    escapeFunction(state, tag.fn);
  }
  markEscaped(state, tag);
}

api.deferExpression = deferExpression;
api.escapeNestedFunctions = escapeNestedFunctions;
api.releaseEscaping = releaseEscaping;

module.exports = {
  callFunction,
  escapeFunction,
  flushPending,
  registerFn,
  visitFunction,
};
