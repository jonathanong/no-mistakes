"use strict";

function isBlank(tag) {
  return tag.type === "unknown";
}

function isTrackable(tag) {
  return tag.type === "fresh" || tag.type === "fn" || tag.type === "union";
}

function addMember(members, tag) {
  if (tag.type === "union") {
    for (const member of tag.members) {
      addMember(members, member);
    }
    return;
  }
  members.push(tag);
}

function unionOf(left, right) {
  const members = [];
  addMember(members, left);
  addMember(members, right);
  return { type: "union", members };
}

// Shared wins over a fresh, null, or unknown side so the join stays reportable.
// Two fresh or function tags stay a union: escaping it releases both, and a
// write reports only when a member is already shared or escaped.
function joinValues(left, right) {
  if (left === right) {
    return left;
  }
  if (isBlank(left)) {
    return right;
  }
  if (isBlank(right)) {
    return left;
  }
  if (left.type === "shared" && right.type === "shared") {
    return left.origin === right.origin ? left : unionOf(left, right);
  }
  if (left.type === "shared") {
    return left;
  }
  if (right.type === "shared") {
    return right;
  }
  if (left.type === "this" && right.type === "this") {
    return left.inClass ? left : right;
  }
  if (isTrackable(left) || isTrackable(right)) {
    return unionOf(left, right);
  }
  if (left.type === "this") {
    return left;
  }
  return right.type === "this" ? right : left;
}

function readOnlyFresh(tag) {
  if (tag.type === "union") {
    return tag.members.every(readOnlyFresh);
  }
  return tag.type === "fresh" && !tag.escaped;
}

module.exports = {
  joinValues,
  readOnlyFresh,
};
