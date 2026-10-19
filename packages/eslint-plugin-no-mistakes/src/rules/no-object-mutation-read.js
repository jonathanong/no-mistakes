"use strict";

const { joinValues } = require("./no-object-mutation-join");

function unknown() {
  return { type: "unknown" };
}

function elementTag(tag) {
  if (tag.type === "union") {
    let joined = unknown();
    for (const member of tag.members) {
      joined = joinValues(joined, elementTag(member));
    }
    return joined;
  }
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

function readMember(tag, key) {
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
  } else if (tag.type === "fresh") {
    // A computed key can name any field. Prefer a shared or already-escaped
    // candidate over the fresh object, which would hide that write.
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

function escapeFresh(tag, seen = new Set()) {
  if (!tag || seen.has(tag)) {
    return;
  }
  if (tag.type === "union") {
    seen.add(tag);
    for (const member of tag.members) {
      escapeFresh(member, seen);
    }
    return;
  }
  if (tag.type !== "fresh") {
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

module.exports = {
  elementTag,
  escapeFresh,
  readMember,
};
