"use strict";

const {
  abruptCompletionReachesMatcher,
  alwaysExits,
  breakSkipsMatcher,
  caughtThrowCanContinue,
  contains,
} = require("./test-no-delayed-rejects-abrupt");

function isLoop(node) {
  return (
    node?.type === "WhileStatement" ||
    node?.type === "DoWhileStatement" ||
    node?.type === "ForStatement" ||
    node?.type === "ForInStatement" ||
    node?.type === "ForOfStatement"
  );
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
        const exit = statements
          .slice(currentIndex + 1, end)
          .find((statement) => alwaysExits(statement) || breakSkipsMatcher(statement, matcher));
        if (
          exit &&
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

function isConditionalBoundary(node) {
  return (
    node.type === "IfStatement" ||
    node.type === "ConditionalExpression" ||
    node.type === "LogicalExpression" ||
    node.type === "SwitchStatement" ||
    node.type === "TryStatement" ||
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
        parent.type === "IfStatement"
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
    if (isConditionalBoundary(parent)) conditional = true;
    current = parent;
  }
  return false;
}

module.exports = { canReachMatcher, contains, executesBefore };
