"use strict";

const { ARRAY_METHODS } = require("./no-object-mutation-ast");
const { joinValues } = require("./no-object-mutation-join");

function unknown() {
  return { type: "unknown" };
}

function elementTag(tag) {
  if (tag.mutator) tag = tag.mutator.member;
  if (tag.type === "union") {
    let joined = unknown();
    for (const member of tag.members) {
      joined = joinValues(joined, elementTag(member));
    }
    return joined;
  }
  if (tag.type === "shared" || tag.type === "this") {
    return tag;
  }
  if (tag.type === "mapset") {
    return tag.contents || unknown();
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

function readMember(tag, key, objectNode) {
  if (tag?.mutator) tag = tag.mutator.member;
  if (objectNode && ARRAY_METHODS.has(key)) {
    const member = readMember(tag, key);
    if (member.type === "fn" || member.fn) return member;
    // Keep the receiver's actual value identity, including later escapes. The
    // separate member tag preserves ordinary object/collection provenance.
    return { type: "unknown", mutator: { receiver: tag, receiverNode: objectNode, member } };
  }
  if (!tag || tag.type === "unknown" || tag.type === "fn") {
    return unknown();
  }
  if (tag.type === "union") {
    let joined = unknown();
    for (const member of tag.members) {
      joined = joinValues(joined, readMember(member, key));
    }
    return joined;
  }
  if (tag.type === "shared" || tag.type === "mapset" || tag.type === "this") {
    return tag;
  }
  if (tag.escaped) {
    return tag;
  }
  if (key) {
    if (tag.fields.has(key)) {
      return tag.fields.get(key);
    }
    for (const item of tag.items) {
      if (item.type === "fn" && item.methodName === key) {
        return item;
      }
    }
  } else {
    // A computed key can name any field. Only an unescaped fresh tag reaches
    // here. Prefer a shared or already-escaped candidate over that object.
    for (const field of tag.fields.values()) {
      if (reportableMember(field)) {
        return field;
      }
    }
    if (reportableMember(tag.dynamic)) {
      return tag.dynamic;
    }
  }
  if (tag.dynamic) {
    return tag.dynamic;
  }
  return tag;
}

function reportableMember(tag) {
  if (tag == null) {
    return false;
  }
  if (tag.type === "shared" || tag.type === "this") {
    return true;
  }
  return tag.type === "fresh" && tag.escaped;
}

function iteratedContents(tag) {
  if (tag.mutator) tag = tag.mutator.member;
  if (tag.type === "unknown" || tag.type === "fn") {
    return unknown();
  }
  if (tag.type === "union") {
    let joined = unknown();
    for (const member of tag.members) {
      joined = joinValues(joined, iteratedContents(member));
    }
    return joined;
  }
  if (tag.type === "shared" || tag.type === "this") {
    return tag;
  }
  if (tag.type === "mapset") {
    return tag.contents || unknown();
  }
  if (tag.escaped) {
    return tag;
  }
  let joined = unknown();
  for (const field of tag.fields.values()) {
    joined = joinValues(joined, field);
  }
  for (const item of tag.items) {
    if (item.type === "fn") {
      continue;
    }
    joined = joinValues(joined, item);
  }
  if (tag.dynamic && tag.dynamic.type !== "unknown") {
    joined = joinValues(joined, tag.dynamic);
  }
  return joined;
}

function escapeFresh(tag, seen = new Set()) {
  if (!tag || seen.has(tag)) {
    return;
  }
  if (tag.mutator) {
    seen.add(tag);
    escapeFresh(tag.mutator.receiver, seen);
    for (const value of tag.mutator.boundArgs || []) escapeFresh(value, seen);
  }
  if (tag.type === "union") {
    seen.add(tag);
    for (const member of tag.members) {
      escapeFresh(member, seen);
    }
  } else if (tag.type === "mapset") {
    // A map that leaves the function can be filled by the caller.
    if (!tag.contents) {
      tag.contents = { type: "shared", origin: "parameter" };
    }
    seen.add(tag);
  } else if (tag.type === "fresh") {
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
}

module.exports = {
  elementTag,
  escapeFresh,
  iteratedContents,
  readMember,
};
