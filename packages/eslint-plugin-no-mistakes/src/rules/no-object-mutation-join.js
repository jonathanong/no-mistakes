"use strict";

function isBlank(tag) {
  return tag.type === "unknown" && tag.scalar;
}

function isTrackable(tag) {
  return tag.type === "fresh" || tag.type === "fn" || tag.type === "union";
}

function addMember(members, tag, seen) {
  if (tag.type === "union") {
    for (const member of tag.members) {
      addMember(members, member, seen);
    }
    return;
  }
  if (!seen.has(tag)) {
    seen.add(tag);
    members.push(tag);
  }
}

function unionOf(left, right) {
  const members = [];
  const seen = new Set();
  addMember(members, left, seen);
  addMember(members, right, seen);
  return { type: "union", members };
}

// Keep value identities so a later escape or method write reaches every alternative.
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
  if (left.type === "shared" || left.type === "unknown" || right.type === "unknown") {
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
    const known = tag.members.filter((member) => member.type !== "unknown");
    return known.length > 0 && known.every(readOnlyFresh);
  }
  return tag.type === "fresh" && !tag.escaped;
}

module.exports = {
  joinValues,
  readOnlyFresh,
};
