"use strict";

function contains(ancestor, node) {
  return ancestor.range[0] <= node.range[0] && ancestor.range[1] >= node.range[1];
}

function alwaysExits(statement) {
  if (statement.type === "ReturnStatement" || statement.type === "ThrowStatement") return true;
  if (statement.type === "BlockStatement") return statement.body.some(alwaysExits);
  if (statement.type === "TryStatement") {
    if (statement.finalizer && alwaysExits(statement.finalizer)) return true;
    if (!alwaysExits(statement.block)) return false;
    return (
      !statement.handler || alwaysReturns(statement.block) || alwaysExits(statement.handler.body)
    );
  }
  return (
    statement.type === "IfStatement" &&
    statement.alternate &&
    alwaysExits(statement.consequent) &&
    alwaysExits(statement.alternate)
  );
}

function alwaysReturns(statement) {
  if (statement.type === "ReturnStatement") return true;
  if (statement.type === "BlockStatement") {
    const exit = statement.body.find(alwaysExits);
    return Boolean(exit && alwaysReturns(exit));
  }
  if (statement.type === "TryStatement") {
    if (statement.finalizer && alwaysExits(statement.finalizer)) {
      return alwaysReturns(statement.finalizer);
    }
    if (alwaysReturns(statement.block)) return true;
    return Boolean(
      statement.handler && alwaysExits(statement.block) && alwaysReturns(statement.handler.body),
    );
  }
  return (
    statement.type === "IfStatement" &&
    statement.alternate &&
    alwaysReturns(statement.consequent) &&
    alwaysReturns(statement.alternate)
  );
}

function alwaysThrows(statement) {
  if (statement.type === "ThrowStatement") return true;
  if (statement.type === "BlockStatement") {
    const exit = statement.body.find(alwaysExits);
    return Boolean(exit && alwaysThrows(exit));
  }
  if (statement.type === "TryStatement") {
    if (statement.finalizer && alwaysExits(statement.finalizer)) {
      return alwaysThrows(statement.finalizer);
    }
    return (
      alwaysThrows(statement.block) && (!statement.handler || alwaysThrows(statement.handler.body))
    );
  }
  return (
    statement.type === "IfStatement" &&
    statement.alternate &&
    alwaysThrows(statement.consequent) &&
    alwaysThrows(statement.alternate)
  );
}

function matcherRunsInClause(node, matcher, clause) {
  let current = node;
  while (current.parent) {
    const parent = current.parent;
    if (parent.type === "TryStatement" && clause(parent, current, matcher)) return true;
    current = parent;
  }
  return false;
}

function abruptCompletionReachesMatcher(node, matcher) {
  const runsInFinally = matcherRunsInClause(
    node,
    matcher,
    (parent, current) =>
      parent.finalizer &&
      (current === parent.block || current === parent.handler) &&
      contains(parent.finalizer, matcher),
  );
  const runsInCatch = matcherRunsInClause(
    node,
    matcher,
    (parent, current) =>
      current === parent.block && parent.handler && contains(parent.handler, matcher),
  );
  return runsInFinally || (alwaysThrows(node) && runsInCatch);
}

function caughtThrowCanContinue(node, matcher) {
  if (!alwaysThrows(node)) return false;
  let current = node;
  while (current.parent) {
    const parent = current.parent;
    if (parent.type === "TryStatement" && current === parent.block) {
      if (parent.finalizer && alwaysExits(parent.finalizer)) {
        if (!alwaysThrows(parent.finalizer)) return false;
        current = parent;
        continue;
      }
      if (parent.handler) {
        if (contains(parent, matcher)) return false;
        if (!alwaysExits(parent.handler.body)) return true;
        if (!alwaysThrows(parent.handler.body)) return false;
        current = parent;
        continue;
      }
    }
    current = parent;
  }
  return false;
}

function directBreakSkipsMatcher(node, matcher) {
  let current = node.parent;
  while (true) {
    const isTarget = node.label
      ? current.type === "LabeledStatement" && current.label.name === node.label.name
      : current.type === "SwitchStatement" ||
        current.type === "WhileStatement" ||
        current.type === "DoWhileStatement" ||
        current.type === "ForStatement" ||
        current.type === "ForInStatement" ||
        current.type === "ForOfStatement";
    if (isTarget) return contains(current, matcher);
    current = current.parent;
  }
}

function breakSkipsMatcher(node, matcher) {
  if (node.type === "BreakStatement") return directBreakSkipsMatcher(node, matcher);
  if (node.type === "BlockStatement") {
    return node.body.some((statement) => breakSkipsMatcher(statement, matcher));
  }
  return Boolean(
    node.type === "IfStatement" &&
    node.alternate &&
    breakSkipsMatcher(node.consequent, matcher) &&
    breakSkipsMatcher(node.alternate, matcher),
  );
}

module.exports = {
  abruptCompletionReachesMatcher,
  alwaysExits,
  breakSkipsMatcher,
  caughtThrowCanContinue,
  contains,
};
