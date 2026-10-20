"use strict";

const { joinValues } = require("./no-object-mutation-join");
const { elementTag, escapeFresh, readMember } = require("./no-object-mutation-read");

function rootEnv() {
  const env = {
    cells: new Map(),
    outer: null,
    isModule: true,
    inClass: false,
    fnTag: null,
    pending: [],
    hasArguments: false,
    namespaceShared: false,
    receiver: null,
    moduleFresh: [],
  };
  env.ownerEnv = env;
  env.functionEnv = env;
  return env;
}

function child(scope) {
  return {
    cells: new Map(),
    outer: scope,
    isModule: false,
    inClass: Boolean(scope.inClass),
    ownerEnv: scope.ownerEnv,
    functionEnv: scope.functionEnv,
    fnTag: scope.fnTag,
    hasArguments: Boolean(scope.hasArguments),
    namespaceShared: Boolean(scope.namespaceShared),
    receiver: scope.receiver || null,
  };
}

function createFresh(scope) {
  return {
    type: "fresh",
    escaped: false,
    fields: new Map(),
    items: [],
    dynamic: null,
    ownerEnv: scope.ownerEnv,
  };
}

function shared(origin) {
  return { type: "shared", origin };
}

function unknown() {
  return { type: "unknown" };
}

function freshCopy(tag, scope) {
  const fresh = createFresh(scope);
  remember(fresh, tag);
  return fresh;
}

function remember(fresh, tag) {
  if (!tag || tag.type === "unknown") {
    if (!fresh.dynamic) {
      fresh.dynamic = unknown();
    }
    return;
  }
  if (tag.type === "union") {
    for (const member of tag.members) {
      remember(fresh, member);
    }
    return;
  }
  fresh.items.push(tag);
  fresh.dynamic = joinDynamic(fresh.dynamic, tag);
}

function joinDynamic(existing, tag) {
  if (!existing || existing.type === "unknown") {
    return tag;
  }
  if (existing === tag) {
    return existing;
  }
  if (existing.type === "shared" && tag.type === "shared" && existing.origin === tag.origin) {
    return existing;
  }
  if (existing.type === "mapset" && tag.type === "mapset") {
    return existing;
  }
  return unknown();
}

function lookup(scope, name) {
  for (let cursor = scope; cursor; cursor = cursor.outer) {
    if (cursor.cells.has(name)) {
      return cursor.cells.get(name);
    }
  }
  return null;
}

function callable(tag) {
  if (!tag) {
    return null;
  }
  if (tag.type === "fn") {
    return tag;
  }
  // Module bindings are the only tags with `fn`, and that value is a function tag.
  if (tag.fn) {
    return tag.fn;
  }
  return null;
}

module.exports = {
  callable,
  child,
  createFresh,
  elementTag,
  escapeFresh,
  freshCopy,
  joinValues,
  lookup,
  readMember,
  remember,
  rootEnv,
  shared,
  unknown,
};
