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

// A shared side stays paired with the other side. Escaping the join then
// releases a fresh or function member, and a write still reports the shared
// one. Same-origin shared tags collapse. Two fresh or function tags stay a
// union for the same reason.
function joinValues(left, right) {
  if (left === right) {
    return left;
  }
  // A choice of method values keeps ordinary object provenance. It does not
  // select either receiver as the callable mutator for the whole expression.
  if (left.mutator) left = left.mutator.member;
  if (right.mutator) right = right.mutator.member;
  if (isBlank(left)) {
    return right;
  }
  if (isBlank(right)) {
    return left;
  }
  if (left.type === "shared") {
    if (right.type === "shared") {
      if (left.origin === right.origin) {
        return left;
      }
      return unionOf(left, right);
    }
    return unionOf(left, right);
  }
  if (right.type === "shared") {
    return unionOf(left, right);
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
  if (tag.mutator) tag = tag.mutator.member;
  if (tag.type === "union") {
    return tag.members.every(readOnlyFresh);
  }
  return tag.type === "fresh" && !tag.escaped;
}

module.exports = {
  joinValues,
  readOnlyFresh,
};
