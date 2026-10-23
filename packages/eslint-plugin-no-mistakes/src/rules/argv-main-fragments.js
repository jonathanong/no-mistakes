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
  // Accepted prefixes have at most eight characters; reject before allocating.
  if (typeof left === "string" && typeof right === "string")
    return left.length + right.length <= 8 ? left + right : 0;
  if (typeof left === "string" && Array.isArray(right))
    return left.length + right[0].length <= 8 ? [left + right[0], right[1]] : 0;
  if (Array.isArray(left) && typeof right === "string") return right === "" ? left : 0;
  return 0;
}

module.exports = { ARGV, identity, concatenate };
