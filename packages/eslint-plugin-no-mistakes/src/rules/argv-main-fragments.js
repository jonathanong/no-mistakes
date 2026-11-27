"use strict";
const { unwrapExpression } = require("./async-ast");
const { literalString } = require("./module-mock-helpers");
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
function fileUrlArgument(target, read) {
  if (target?.type !== "NewExpression" || target.arguments.length !== 1) return 0;
  const argument = read(target.arguments[0]);
  return Array.isArray(argument) && identity(argument) === ARGV ? argument : 0;
}
function staticIndexOne(node) {
  const current = unwrapExpression(node);
  if (!current) return false;
  if (current.type === "Literal") return current.value === 1 || current.value === "1";
  return literalString(current) === "1";
}
function objectPatternKey(pattern, name) {
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const value =
      property.value.type === "AssignmentPattern" ? property.value.left : property.value;
    if (value.type !== "Identifier" || value.name !== name) continue;
    const key = unwrapExpression(property.key);
    if (!property.computed && key?.type === "Identifier") return key.name;
    if (key?.type === "Literal") return String(key.value);
    return literalString(key);
  }
  return null;
}
function isProcessSpecifier(node) {
  return ["process", "node:process"].includes(literalString(unwrapExpression(node)));
}
function isProcessRequire(node, context, isLocalRequire) {
  const callee = unwrapExpression(node?.callee);
  return (
    node?.type === "CallExpression" &&
    callee?.type === "Identifier" &&
    callee.name === "require" &&
    !isLocalRequire(callee, context) &&
    isProcessSpecifier(node.arguments?.[0])
  );
}
module.exports = {
  ARGV,
  identity,
  concatenate,
  fileUrlArgument,
  staticIndexOne,
  objectPatternKey,
  isProcessSpecifier,
  isProcessRequire,
};
