"use strict";

const {
  alwaysExits,
  alwaysThrows,
  breakSkipsMatcher,
  continueSkipsMatcher,
} = require("./test-no-delayed-rejects-abrupt");

function contains(ancestor, node) {
  return ancestor.range[0] <= node.range[0] && ancestor.range[1] >= node.range[1];
}

function isFunction(node) {
  return (
    node.type === "ArrowFunctionExpression" ||
    node.type === "FunctionDeclaration" ||
    node.type === "FunctionExpression"
  );
}

function isLoop(node) {
  return (
    node.type === "WhileStatement" ||
    node.type === "DoWhileStatement" ||
    node.type === "ForStatement" ||
    node.type === "ForInStatement" ||
    node.type === "ForOfStatement"
  );
}

function jumpTarget(node) {
  let current = node.parent;
  while (true) {
    const isTarget = node.label
      ? current.type === "LabeledStatement" && current.label.name === node.label.name
      : isLoop(current) || (node.type === "BreakStatement" && current.type === "SwitchStatement");
    if (isTarget) return current;
    current = current.parent;
  }
}

function hasEnclosingLoop(node) {
  let current = node.parent;
  while (current) {
    if (isLoop(current)) return true;
    current = current.parent;
  }
  return false;
}

function breakCanReachLaterIteration(target, resourceLoop) {
  if (target === resourceLoop) return hasEnclosingLoop(resourceLoop);
  return (
    target.type === "LabeledStatement" && contains(target, resourceLoop) && hasEnclosingLoop(target)
  );
}

const BACKEDGE = 1;
const FALLTHROUGH = 0;
const STOP = -1;

function branchOutcome(statement, matcher) {
  if (statement.type === "BlockStatement" || statement.type === "SwitchCase") {
    const statements = statement.type === "BlockStatement" ? statement.body : statement.consequent;
    for (const child of statements) {
      const outcome = branchOutcome(child, matcher);
      if (outcome !== FALLTHROUGH) return outcome;
    }
    return FALLTHROUGH;
  }
  if (statement.type === "IfStatement" && statement.alternate) {
    const consequent = branchOutcome(statement.consequent, matcher);
    const alternate = branchOutcome(statement.alternate, matcher);
    if (consequent === BACKEDGE || alternate === BACKEDGE) return BACKEDGE;
    return consequent === STOP && alternate === STOP ? STOP : FALLTHROUGH;
  }
  if (continueSkipsMatcher(statement, matcher)) return BACKEDGE;
  if (alwaysExits(statement) || breakSkipsMatcher(statement, matcher)) return STOP;
  return FALLTHROUGH;
}

function canReachEnclosingBackedge(node, resourceLoop, matcher) {
  let current = node;
  while (current.parent) {
    const parent = current.parent;
    const statements =
      parent.type === "BlockStatement"
        ? parent.body
        : parent.type === "SwitchCase"
          ? parent.consequent
          : null;
    if (statements) {
      const following = statements.slice(statements.indexOf(current) + 1);
      for (const statement of following) {
        const outcome = branchOutcome(statement, matcher);
        if (outcome === BACKEDGE) return true;
        if (outcome === STOP) return false;
      }
    }
    if (isLoop(parent) && contains(parent, resourceLoop)) return true;
    current = parent;
  }
  return false;
}

function throwCanReachLaterIteration(node, matcher, resourceLoop) {
  let current = node;
  while (current.parent) {
    const parent = current.parent;
    if (
      parent.type === "TryStatement" &&
      current !== parent.finalizer &&
      parent.finalizer &&
      alwaysExits(parent.finalizer)
    ) {
      if (!alwaysThrows(parent.finalizer) || !contains(parent, resourceLoop)) return false;
      current = parent;
      continue;
    }
    if (parent.type === "TryStatement" && current === parent.block && parent.handler) {
      if (!contains(parent, resourceLoop)) return false;
      if (continueSkipsMatcher(parent.handler.body, matcher)) return true;
      if (breakSkipsMatcher(parent.handler.body, matcher)) return false;
      if (!alwaysExits(parent.handler.body)) {
        return canReachEnclosingBackedge(parent, resourceLoop, matcher);
      }
      if (!alwaysThrows(parent.handler.body)) return false;
      current = parent;
      continue;
    }
    current = parent;
  }
  return false;
}

function jumpExitsToLaterIteration(node, resourceLoop) {
  const target = jumpTarget(node);
  if (node.type === "ContinueStatement") {
    return target === resourceLoop || contains(target, resourceLoop);
  }
  return breakCanReachLaterIteration(target, resourceLoop);
}

function possibleResourceExitBeforeMatcher(node, matcher, resourceLoop) {
  if (node.range[0] >= matcher.range[0] || (node !== matcher && isFunction(node))) return false;
  if (node.type === "ContinueStatement" || node.type === "BreakStatement") {
    return jumpExitsToLaterIteration(node, resourceLoop);
  }
  if (node.type === "ThrowStatement") {
    return throwCanReachLaterIteration(node, matcher, resourceLoop);
  }
  for (const [key, value] of Object.entries(node)) {
    if (key === "parent") continue;
    const children = Array.isArray(value) ? value : [value];
    if (
      children.some(
        (child) => child?.type && possibleResourceExitBeforeMatcher(child, matcher, resourceLoop),
      )
    ) {
      return true;
    }
  }
  return false;
}

module.exports = { possibleResourceExitBeforeMatcher, branchOutcome };
