"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { findContainingRoot, traverse } = require("./async-ast");
const { originDeclarator } = require("./test-no-delayed-rejects-promise-alias");
const { chainIsSafelyObserved, promiseChainBase } = require("./test-no-delayed-rejects-chains");
const { canReachMatcher, contains, executesBefore } = require("./test-no-delayed-rejects-flow");
const { isImmediateObserver } = require("./test-no-delayed-rejects-observers");
const {
  awaitUsingScope,
  promiseExistsBeforeInitializerSuspension,
  suspensionOccursBeforeMatcher,
} = require("./test-no-delayed-rejects-suspensions");

const { expectedIdentifier } = require("./test-no-delayed-rejects-expect");

function matcherCall(rejects) {
  let current = rejects;
  while (
    current.parent?.type === "ChainExpression" ||
    (current.parent?.type === "MemberExpression" && current.parent.object === current)
  ) {
    current = current.parent;
  }
  return current.parent?.type === "CallExpression" && current.parent.callee === current
    ? current.parent
    : null;
}

function isSameConst(identifier, declarator, context) {
  return identifier.type === "Identifier" && originDeclarator(identifier, context) === declarator;
}

function isRejectionHandlerCall(node, declarator, context) {
  if (node.type !== "CallExpression" || node.callee.type !== "MemberExpression") return false;
  const object = promiseChainBase(node.callee.object);
  if (!isSameConst(object, declarator, context)) return false;
  return chainIsSafelyObserved(node);
}

function rejectionMatcherCall(node, declarator, context) {
  if (node.type !== "MemberExpression") return null;
  const identifier = expectedIdentifier(node, context);
  if (!identifier || !isSameConst(identifier, declarator, context)) return null;
  return matcherCall(node);
}

function executesWhenDeclared(node, functionNode) {
  let current = node;
  while (current.parent && current.parent !== functionNode) {
    const parent = current.parent;
    if (
      parent.type === "PropertyDefinition" &&
      !parent.static &&
      parent.value &&
      contains(parent.value, node)
    ) {
      return false;
    }
    current = parent;
  }
  return true;
}

function collectObserverSites(context, functionNode, declarator) {
  const observers = [];
  traverse(context, functionNode, (node) => {
    const matcher = rejectionMatcherCall(node, declarator, context);
    const handler =
      isRejectionHandlerCall(node, declarator, context) ||
      isImmediateObserver(node, declarator, context, isSameConst);
    if (handler && executesWhenDeclared(node, functionNode)) observers.push(node);
    else if (matcher && executesWhenDeclared(matcher, functionNode)) observers.push(matcher);
  });
  return observers;
}

function hasObserverBeforeSuspension(observers, suspension) {
  return observers.some((observer) => {
    const observesAfterForAwaitSuspends =
      suspension.type === "ForOfStatement" &&
      (contains(suspension.left, observer) || contains(suspension.body, observer));
    return !observesAfterForAwaitSuspends && executesBefore(observer, suspension);
  });
}

function hasInterveningAwait(context, declarator, matcher, observerCache) {
  const functionNode = findContainingRoot(declarator);
  if (!functionNode || functionNode !== findContainingRoot(matcher)) return false;
  let observers = observerCache.get(declarator);
  if (!observers) {
    observers = collectObserverSites(context, functionNode, declarator);
    observerCache.set(declarator, observers);
  }

  let found = false;
  traverse(context, functionNode, (node) => {
    const suspension = awaitUsingScope(node, matcher) ?? node;
    if (
      (suspension !== node || suspensionOccursBeforeMatcher(node, matcher, functionNode)) &&
      (node.range[0] >= declarator.init.range[1] ||
        promiseExistsBeforeInitializerSuspension(declarator.init, node)) &&
      canReachMatcher(suspension, matcher, functionNode) &&
      !hasObserverBeforeSuspension(observers, suspension)
    ) {
      found = true;
    }
  });
  return found;
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require rejection observers before coordination awaits",
      recommended: false,
    },
    schema: [],
    messages: {
      delayedReject:
        "Attach a rejection observer to this const promise before awaiting coordination; the promise can reject before expect(...).rejects observes it.",
    },
  },
  (context) => {
    const observerCache = new WeakMap();
    const suppressed = ruleSuppression(context, "test-no-delayed-rejects");
    return {
      MemberExpression(node) {
        const identifier = expectedIdentifier(node, context);
        if (!identifier) return;
        const declarator = originDeclarator(identifier, context);
        const call = matcherCall(node);
        if (
          !declarator ||
          !call ||
          suppressed(node) ||
          suppressed(call) ||
          !hasInterveningAwait(context, declarator, call, observerCache)
        ) {
          return;
        }
        context.report({ node, messageId: "delayedReject" });
      },
    };
  },
);
