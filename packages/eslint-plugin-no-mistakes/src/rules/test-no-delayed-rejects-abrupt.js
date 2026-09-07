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
      } else if (parent.handler) {
        if (contains(parent, matcher)) return false;
        if (!alwaysExits(parent.handler.body)) return true;
        if (!alwaysThrows(parent.handler.body)) return false;
      }
    }
    current = parent;
  }
  return false;
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

function directJumpSkipsMatcher(node, matcher) {
  let current = node.parent;
  while (true) {
    const isTarget = node.label
      ? current.type === "LabeledStatement" && current.label.name === node.label.name
      : isLoop(current) || (node.type === "BreakStatement" && current.type === "SwitchStatement");
    if (isTarget) return contains(current, matcher);
    current = current.parent;
  }
}

function jumpSkipsMatcher(node, matcher, type) {
  if (node.type === type) return directJumpSkipsMatcher(node, matcher);
  if (node.type === "BlockStatement" || node.type === "SwitchCase") {
    const statements = node.type === "BlockStatement" ? node.body : node.consequent;
    return statements.some((statement) => jumpSkipsMatcher(statement, matcher, type));
  }
  if (node.type === "TryStatement") {
    if (node.finalizer && jumpSkipsMatcher(node.finalizer, matcher, type)) return true;
    if (node.finalizer && alwaysExits(node.finalizer)) {
      return jumpSkipsMatcher(node.finalizer, matcher, type);
    }
    if (jumpSkipsMatcher(node.block, matcher, type)) return true;
    return Boolean(
      node.handler &&
      alwaysThrows(node.block) &&
      jumpSkipsMatcher(node.handler.body, matcher, type),
    );
  }
  return Boolean(
    node.type === "IfStatement" &&
    node.alternate &&
    jumpSkipsMatcher(node.consequent, matcher, type) &&
    jumpSkipsMatcher(node.alternate, matcher, type),
  );
}

function breakSkipsMatcher(node, matcher) {
  return jumpSkipsMatcher(node, matcher, "BreakStatement");
}

function continueSkipsMatcher(node, matcher) {
  return jumpSkipsMatcher(node, matcher, "ContinueStatement");
}

module.exports = {
  abruptCompletionReachesMatcher,
  alwaysExits,
  alwaysThrows,
  breakSkipsMatcher,
  continueSkipsMatcher,
  caughtThrowCanContinue,
  contains,
};
