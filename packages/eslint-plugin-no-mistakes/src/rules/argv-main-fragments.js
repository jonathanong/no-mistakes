"use strict";
const ARGV = 1;
// Strings and [prefix, suffix] fragments stay separate from path identities.
// Keeping the fragment until the comparison lets nested concatenations fold once.
function identity(value) {
  if (Array.isArray(value)) return /^file:\/\/\/?$/i.test(value[0]) && value[1] === "" ? ARGV : 0;
  return typeof value === "number" ? value : 0;
}
function concatenate(left, right) {
  if (left === ARGV) left = ["", ""];
  if (right === ARGV) right = ["", ""];
  if (typeof left === "string" && typeof right === "string") return left + right;
  if (typeof left === "string" && Array.isArray(right)) return [left + right[0], right[1]];
  if (Array.isArray(left) && typeof right === "string") return [left[0], left[1] + right];
  return 0;
}

module.exports = { ARGV, identity, concatenate };
