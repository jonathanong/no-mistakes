"use strict";

const { contains } = require("./test-no-delayed-rejects-flow");
const { isPromiseChainMember } = require("./test-no-delayed-rejects-chains");
const { possibleResourceExitBeforeMatcher } = require("./test-no-delayed-rejects-loop-jumps");

function isLoop(node) {
  return (
    node?.type === "WhileStatement" ||
    node?.type === "DoWhileStatement" ||
    node?.type === "ForStatement" ||
    node?.type === "ForInStatement" ||
    node?.type === "ForOfStatement"
  );
}

function commonLoop(first, second, functionNode) {
  let current = first.parent;
  while (current && current !== functionNode) {
    if (isLoop(current) && contains(current, second)) return current;
    current = current.parent;
  }
  return null;
}

function loopBranchesCanReorder(suspension, matcher, functionNode) {
  const loop = commonLoop(suspension, matcher, functionNode);
  if (!loop) return false;
  let current = suspension;
  while (current && current !== loop) {
    const parent = current.parent;
    if (parent?.type === "IfStatement" || parent?.type === "ConditionalExpression") {
      if (current === parent.alternate && contains(parent.consequent, matcher)) return true;
    }
    if (current.type === "SwitchCase" && parent?.type === "SwitchStatement") {
      return parent.cases.some((item) => item !== current && contains(item, matcher));
    }
    current = parent;
  }
  current = matcher;
  while (current && current !== loop) {
    const parent = current.parent;
    if (
      ((parent?.type === "IfStatement" || parent?.type === "ConditionalExpression") &&
        (current === parent.consequent || current === parent.alternate)) ||
      (parent?.type === "LogicalExpression" && current === parent.right) ||
      current.type === "SwitchCase"
    ) {
      return true;
    }
    current = parent;
  }
  return false;
}

function suspensionOccursBeforeMatcher(node, matcher, functionNode) {
  if (
    node.type === "VariableDeclaration" &&
    node.kind === "await using" &&
    node.parent.type === "ForOfStatement" &&
    node.parent.left === node &&
    contains(node.parent.body, matcher)
  ) {
    return (
      loopBranchesCanReorder(node, matcher, functionNode) ||
      possibleResourceExitBeforeMatcher(node.parent.body, matcher, node.parent)
    );
  }
  let isSuspension = node.type === "AwaitExpression" || node.type === "YieldExpression";
  if (node.type === "ForOfStatement") {
    isSuspension =
      node.await &&
      (!contains(node, matcher) || contains(node.left, matcher) || contains(node.body, matcher));
  } else if (isSuspension && contains(node, matcher)) {
    isSuspension = false;
  }
  if (!isSuspension) return false;
  return node.range[0] < matcher.range[1] || loopBranchesCanReorder(node, matcher, functionNode);
}

function promiseExistsBeforeInitializerSuspension(initializer, suspension) {
  let current = suspension;
  while (current && current !== initializer) {
    const parent = current.parent;
    if (
      parent?.type === "CallExpression" &&
      parent.arguments.includes(current) &&
      isPromiseChainMember(parent.callee) &&
      parent.callee.object.range[1] <= suspension.range[0]
    ) {
      return true;
    }
    current = parent;
  }
  return false;
}

function awaitUsingScope(node, matcher) {
  if (node.type !== "VariableDeclaration" || node.kind !== "await using") return null;
  const parent = node.parent;
  let scope = null;
  if (parent.type === "ForOfStatement" && parent.left === node) scope = parent.body;
  else if (parent.type === "ForStatement" && parent.init === node) scope = parent;
  else {
    let current = parent;
    while (current && current.type !== "BlockStatement") {
      current = current.parent;
    }
    scope = current;
  }
  return scope && scope.range[1] <= matcher.range[0] ? scope : null;
}

module.exports = {
  awaitUsingScope,
  promiseExistsBeforeInitializerSuspension,
  suspensionOccursBeforeMatcher,
};
