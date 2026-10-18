"use strict";

function rootEnv() {
  const env = {
    cells: new Map(),
    outer: null,
    isModule: true,
    inClass: false,
    fnTag: null,
    pending: [],
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

function joinValues(left, right) {
  if (!left || !right || left.type === "unknown" || right.type === "unknown") {
    return unknown();
  }
  if (left === right) {
    return left;
  }
  if (left.type === "shared" && right.type === "shared" && left.origin === right.origin) {
    return left;
  }
  if (left.type === "mapset" && right.type === "mapset") {
    return left;
  }
  if (left.type === "this" && right.type === "this" && left.inClass === right.inClass) {
    return left;
  }
  return unknown();
}

function elementTag(tag) {
  if (tag.type === "shared" || tag.type === "mapset" || tag.type === "this") {
    return tag;
  }
  if (tag.type !== "fresh") {
    return unknown();
  }
  if (tag.escaped) {
    return tag;
  }
  if (tag.dynamic) {
    return tag.dynamic;
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

function readMember(tag, key) {
  if (!tag || tag.type === "unknown" || tag.type === "fn") {
    return unknown();
  }
  if (tag.type === "shared" || tag.type === "mapset" || tag.type === "this") {
    return tag;
  }
  if (tag.escaped) {
    return tag;
  }
  if (key && tag.fields.has(key)) {
    return tag.fields.get(key);
  }
  if (tag.dynamic) {
    return tag.dynamic;
  }
  return tag;
}

function escapeFresh(tag, seen = new Set()) {
  if (!tag || seen.has(tag) || tag.type !== "fresh") {
    return;
  }
  seen.add(tag);
  tag.escaped = true;
  for (const field of tag.fields.values()) {
    escapeFresh(field, seen);
  }
  for (const item of tag.items) {
    escapeFresh(item, seen);
  }
  escapeFresh(tag.dynamic, seen);
}

function callable(tag) {
  if (!tag) {
    return null;
  }
  if (tag.type === "fn") {
    return tag;
  }
  if (tag.fn?.type === "fn") {
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
