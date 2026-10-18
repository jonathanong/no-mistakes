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
  fnEnv.inClass = Boolean(scope.inClass);
  fnEnv.pending = [];
  for (const param of node.params) {
    bindPattern(state, param, shared("parameter"), fnEnv, "param");
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

function visitClass(state, body, scope) {
  const classEnv = child(scope);
  classEnv.inClass = true;
  for (const element of body.body) {
    if (element.computed) {
      api.visit(state, element.key, classEnv, "read");
    }
    for (const decorator of element.decorators ?? []) {
      api.visit(state, decorator.expression, classEnv, "escape");
    }
    if (element.type === "StaticBlock") {
      api.visit(state, element, classEnv, "read");
    } else if (element.value) {
      api.visit(state, element.value, classEnv, "read");
    } else {
      continue;
    }
  }
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

module.exports = {
  callFunction,
  escapeFunction,
  flushPending,
  registerFn,
  visitClass,
  visitFunction,
};
