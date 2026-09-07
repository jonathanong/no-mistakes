"use strict";

function contains(ancestor, node) {
  return ancestor.range[0] <= node.range[0] && ancestor.range[1] >= node.range[1];
}

function alwaysExits(statement) {
  if (statement.type === "ReturnStatement" || statement.type === "ThrowStatement") return true;
  if (statement.type === "BlockStatement") return statement.body.some(alwaysExits);
  return (
    statement.type === "IfStatement" &&
    statement.alternate &&
    alwaysExits(statement.consequent) &&
    alwaysExits(statement.alternate)
  );
}

function matcherRunsInFinally(node, matcher) {
  let current = node;
  while (current.parent) {
    const parent = current.parent;
    if (
      parent.type === "TryStatement" &&
      parent.finalizer &&
      (current === parent.block || current === parent.handler) &&
      contains(parent.finalizer, matcher)
    ) {
      return true;
    }
    current = parent;
  }
  return false;
}

function matcherRunsInCatch(node, matcher) {
  let current = node;
  while (current.parent) {
    const parent = current.parent;
    if (
      parent.type === "TryStatement" &&
      current === parent.block &&
      parent.handler &&
      contains(parent.handler, matcher)
    ) {
      return true;
    }
    current = parent;
  }
  return false;
}

function abruptCompletionReachesMatcher(node, matcher) {
  return (
    matcherRunsInFinally(node, matcher) ||
    (node.type === "ThrowStatement" && matcherRunsInCatch(node, matcher))
  );
}

function branchesAreExclusive(current, parent, matcher, suspension) {
  if (parent?.type === "IfStatement" || parent?.type === "ConditionalExpression") {
    return (
      (current === parent.consequent && parent.alternate && contains(parent.alternate, matcher)) ||
      (current === parent.alternate && contains(parent.consequent, matcher))
    );
  }
  if (current.type === "SwitchCase" && parent?.type === "SwitchStatement") {
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
      !abruptCompletionReachesMatcher(current, matcher)
    ) {
      return false;
    }
    const parent = current.parent;
    if (branchesAreExclusive(current, parent, matcher, suspension)) return false;
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
        const exit = statements.slice(currentIndex + 1, end).find(alwaysExits);
        if (exit && !abruptCompletionReachesMatcher(exit, matcher)) {
          return false;
        }
      }
    }
    current = parent;
  }
  return true;
}

function executionSite(node) {
  let current = node;
  let conditional = false;
  while (current.parent) {
    const parent = current.parent;
    if (parent.type === "BlockStatement" || parent.type === "SwitchCase") {
      return { conditional, container: parent, statement: current };
    }
    if (
      parent.type === "IfStatement" ||
      parent.type === "ConditionalExpression" ||
      parent.type === "LogicalExpression" ||
      parent.type === "SwitchStatement" ||
      parent.type === "TryStatement" ||
      parent.type === "WhileStatement" ||
      parent.type === "DoWhileStatement" ||
      parent.type === "ForStatement" ||
      parent.type === "ForInStatement" ||
      parent.type === "ForOfStatement"
    ) {
      conditional = true;
    }
    current = parent;
  }
  return null;
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
  const observerSite = executionSite(observer);
  if (!observerSite || observerSite.conditional) return false;
  let suspensionStatement = suspension;
  while (suspensionStatement.parent && suspensionStatement.parent !== observerSite.container) {
    suspensionStatement = suspensionStatement.parent;
  }
  if (suspensionStatement.parent !== observerSite.container) return false;
  const statements =
    observerSite.container.type === "BlockStatement"
      ? observerSite.container.body
      : observerSite.container.consequent;
  return statements.indexOf(observerSite.statement) < statements.indexOf(suspensionStatement);
}

module.exports = { canReachMatcher, contains, executesBefore };
