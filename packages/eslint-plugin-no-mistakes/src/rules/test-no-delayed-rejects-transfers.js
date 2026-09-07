"use strict";

const { alwaysExits, breakSkipsMatcher, contains } = require("./test-no-delayed-rejects-abrupt");

function mayThrow(statement) {
  if (statement.type === "ThrowStatement") return true;
  if (statement.type === "BlockStatement" || statement.type === "SwitchCase") {
    for (const child of statement.type === "BlockStatement"
      ? statement.body
      : statement.consequent) {
      if (mayThrow(child)) return true;
      if (alwaysExits(child)) return false;
    }
    return false;
  }
  if (statement.type === "IfStatement") {
    return (
      mayThrow(statement.consequent) ||
      Boolean(statement.alternate && mayThrow(statement.alternate))
    );
  }
  if (statement.type === "SwitchStatement") return statement.cases.some(mayThrow);
  if (statement.type === "LabeledStatement") return mayThrow(statement.body);
  if (
    statement.type === "WhileStatement" ||
    statement.type === "DoWhileStatement" ||
    statement.type === "ForStatement" ||
    statement.type === "ForInStatement" ||
    statement.type === "ForOfStatement"
  ) {
    return mayThrow(statement.body);
  }
  if (statement.type !== "TryStatement") return false;
  if (statement.finalizer && mayThrow(statement.finalizer)) return true;
  if (statement.finalizer && alwaysExits(statement.finalizer)) return false;
  if (!statement.handler) return mayThrow(statement.block);
  return mayThrow(statement.block) && mayThrow(statement.handler.body);
}

function possibleCaughtThrowCanContinue(node, matcher) {
  if (node.type !== "TryStatement" || !node.handler || !mayThrow(node.block)) return false;
  if (contains(node.handler, matcher)) return true;
  return Boolean(
    !contains(node, matcher) &&
    !alwaysExits(node.handler.body) &&
    !breakSkipsMatcher(node.handler.body, matcher) &&
    (!node.finalizer ||
      (!alwaysExits(node.finalizer) && !breakSkipsMatcher(node.finalizer, matcher))),
  );
}

function thrownCompletionCanReachMatcher(origin, matcher) {
  let current = origin;
  while (current.parent) {
    const parent = current.parent;
    if (parent.type === "TryStatement") {
      if (current === parent.block && parent.handler && contains(parent.handler, matcher)) {
        return true;
      }
      if (current !== parent.finalizer && parent.finalizer && contains(parent.finalizer, matcher)) {
        return true;
      }
      if (
        current !== parent.finalizer &&
        parent.finalizer &&
        breakSkipsMatcher(parent.finalizer, matcher)
      ) {
        return false;
      }
      if (current !== parent.block) {
        if (current === parent.handler && parent.finalizer && alwaysExits(parent.finalizer)) {
          if (!mayThrow(parent.finalizer)) return false;
        }
        current = parent;
        continue;
      }
      if (parent.finalizer && alwaysExits(parent.finalizer)) {
        if (!mayThrow(parent.finalizer)) return false;
        current = parent;
        continue;
      }
      if (parent.handler) {
        if (contains(parent, matcher)) return false;
        if (breakSkipsMatcher(parent.handler.body, matcher)) return false;
        if (!alwaysExits(parent.handler.body)) return true;
        if (!mayThrow(parent.handler.body)) return false;
        current = parent;
        continue;
      }
    }
    current = parent;
  }
  return false;
}

module.exports = { mayThrow, possibleCaughtThrowCanContinue, thrownCompletionCanReachMatcher };
