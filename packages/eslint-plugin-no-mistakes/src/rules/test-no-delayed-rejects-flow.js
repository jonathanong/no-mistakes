"use strict";

const {
  abruptCompletionReachesMatcher,
  alwaysExits,
  breakSkipsMatcher,
  caughtThrowCanContinue,
  contains,
} = require("./test-no-delayed-rejects-abrupt");
const {
  possibleCaughtThrowCanContinue,
  suspensionFailureCanReachMatcher,
} = require("./test-no-delayed-rejects-transfers");

function isLoop(node) {
  return (
    node?.type === "WhileStatement" ||
    node?.type === "DoWhileStatement" ||
    node?.type === "ForStatement" ||
    node?.type === "ForInStatement" ||
    node?.type === "ForOfStatement"
  );
}

function isOptionalCall(node) {
  if (node.type !== "CallExpression") return false;
  if (node.optional) return true;
  let current = node.callee;
  while (current.type === "MemberExpression") {
    if (current.optional) return true;
    current = current.object;
  }
  return false;
}

function hasLoopBackedge(node, functionNode) {
  let current = node.parent;
  while (current && current !== functionNode) {
    if (isLoop(current)) return true;
    current = current.parent;
  }
  return false;
}

function branchesAreExclusive(current, parent, matcher, suspension, functionNode) {
  if (parent?.type === "IfStatement" || parent?.type === "ConditionalExpression") {
    if (hasLoopBackedge(parent, functionNode)) return false;
    return (
      (current === parent.consequent && parent.alternate && contains(parent.alternate, matcher)) ||
      (current === parent.alternate && contains(parent.consequent, matcher))
    );
  }
  if (current.type === "SwitchCase" && parent?.type === "SwitchStatement") {
    if (hasLoopBackedge(parent, functionNode)) return false;
    const currentIndex = parent.cases.indexOf(current);
    const matcherIndex = parent.cases.findIndex((item) => contains(item, matcher));
    if (matcherIndex === -1 || matcherIndex === currentIndex) return false;
    if (matcherIndex < currentIndex) return true;
    const suspensionIndex = current.consequent.findIndex((item) => contains(item, suspension));
    return current.consequent
      .slice(suspensionIndex + 1)
      .some((item) => item.type === "BreakStatement" || alwaysExits(item));
  }
  return false;
}

function canReachMatcher(suspension, matcher, functionNode) {
  if (suspensionFailureCanReachMatcher(suspension, matcher)) return true;
  let current = suspension;
  while (current && current !== functionNode) {
    if (
      (current.type === "ReturnStatement" || current.type === "ThrowStatement") &&
      !contains(current, matcher) &&
      !abruptCompletionReachesMatcher(current, matcher) &&
      !caughtThrowCanContinue(current, matcher)
    ) {
      return false;
    }
    if (!contains(current, matcher) && breakSkipsMatcher(current, matcher)) return false;
    const parent = current.parent;
    if (branchesAreExclusive(current, parent, matcher, suspension, functionNode)) return false;
    const statements =
      parent?.type === "BlockStatement"
        ? parent.body
        : parent?.type === "SwitchCase"
          ? parent.consequent
          : null;
    if (statements) {
      const currentIndex = statements.indexOf(current);
      if (currentIndex !== -1) {
        const matcherIndex = statements.findIndex((statement) => contains(statement, matcher));
        const end = matcherIndex === -1 ? statements.length : matcherIndex;
        const following = statements.slice(currentIndex + 1, end);
        const exitIndex = following.findIndex(
          (statement) => alwaysExits(statement) || breakSkipsMatcher(statement, matcher),
        );
        const exit = following[exitIndex];
        const caughtThrow =
          exit &&
          following
            .slice(0, exitIndex + 1)
            .some((statement) => possibleCaughtThrowCanContinue(statement, matcher));
        if (
          exit &&
          !caughtThrow &&
          !abruptCompletionReachesMatcher(exit, matcher) &&
          !caughtThrowCanContinue(exit, matcher)
        ) {
          return false;
        }
      }
    }
    current = parent;
  }
  return true;
}

function isConditionalBoundary(node, child) {
  return (
    node.type === "IfStatement" ||
    node.type === "ConditionalExpression" ||
    node.type === "LogicalExpression" ||
    node.type === "SwitchStatement" ||
    isOptionalCall(node) ||
    (node.type === "TryStatement" && child !== node.finalizer) ||
    isLoop(node)
  );
}

function statementsFor(container) {
  return container.type === "BlockStatement" ? container.body : container.consequent;
}

function directChildIn(node, container) {
  let current = node;
  while (current.parent && current.parent !== container) current = current.parent;
  return current.parent === container ? current : null;
}

function executesBefore(observer, suspension) {
  if (contains(suspension, observer)) {
    let current = observer;
    while (current !== suspension) {
      const parent = current.parent;
      if (
        parent.type === "ConditionalExpression" ||
        parent.type === "LogicalExpression" ||
        parent.type === "IfStatement" ||
        isOptionalCall(parent)
      ) {
        return false;
      }
      current = parent;
    }
    return true;
  }
  let current = observer;
  let conditional = false;
  while (current.parent) {
    const parent = current.parent;
    if ((parent.type === "BlockStatement" || parent.type === "SwitchCase") && !conditional) {
      const suspensionStatement = directChildIn(suspension, parent);
      if (suspensionStatement) {
        const statements = statementsFor(parent);
        if (statements.indexOf(current) < statements.indexOf(suspensionStatement)) return true;
      }
    }
    if (isConditionalBoundary(parent, current)) conditional = true;
    current = parent;
  }
  return false;
}

module.exports = { canReachMatcher, contains, executesBefore };
