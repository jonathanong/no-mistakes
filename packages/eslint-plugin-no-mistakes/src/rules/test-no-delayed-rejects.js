"use strict";

const { rule } = require("../helpers");
const { findContainingFunction, traverse, unwrapExpression } = require("./async-ast");
const {
  chainIsSafelyObserved,
  literalPropertyName,
  promiseChainBase,
} = require("./test-no-delayed-rejects-chains");
const { canReachMatcher, contains, executesBefore } = require("./test-no-delayed-rejects-flow");
const { isImmediateObserver } = require("./test-no-delayed-rejects-observers");
const {
  awaitUsingScope,
  promiseExistsBeforeInitializerSuspension,
  suspensionOccursBeforeMatcher,
} = require("./test-no-delayed-rejects-suspensions");

const EXPECT_MODULES = new Set(["vitest", "@jest/globals"]);

function findVariable(scope, name) {
  let current = scope;
  while (current) {
    const variable =
      current.set?.get?.(name) ?? current.variables?.find((item) => item.name === name);
    if (variable) return variable;
    current = current.upper;
  }
  return null;
}

function isOrdinaryExpect(identifier, context) {
  if (identifier.type !== "Identifier" || identifier.name !== "expect") return false;
  const variable = findVariable(context.sourceCode.getScope(identifier), identifier.name);
  if (!variable || variable.defs.length === 0) return true;
  return variable.defs.every(
    (definition) =>
      definition.type === "ImportBinding" &&
      EXPECT_MODULES.has(definition.parent?.source?.value) &&
      definition.node?.imported?.name === "expect",
  );
}

function isExpectCallee(callee, context) {
  if (isOrdinaryExpect(callee, context)) return true;
  return (
    callee.type === "MemberExpression" &&
    !callee.computed &&
    callee.property.type === "Identifier" &&
    callee.property.name === "soft" &&
    isOrdinaryExpect(callee.object, context)
  );
}

function expectedIdentifier(rejects, context) {
  if (literalPropertyName(rejects) !== "rejects" || rejects.object.type !== "CallExpression") {
    return null;
  }
  const expectation = rejects.object;
  if (expectation.arguments.length !== 1 || !isExpectCallee(expectation.callee, context))
    return null;
  const [argument] = expectation.arguments;
  const unwrapped = unwrapExpression(argument);
  return unwrapped.type === "Identifier" ? unwrapped : null;
}

function matcherCall(rejects) {
  let current = rejects;
  while (current.parent?.type === "MemberExpression" && current.parent.object === current) {
    current = current.parent;
  }
  return current.parent?.type === "CallExpression" && current.parent.callee === current
    ? current.parent
    : null;
}

function constDeclarator(identifier, context) {
  const variable = findVariable(context.sourceCode.getScope(identifier), identifier.name);
  if (!variable || variable.defs.length !== 1) return null;
  const [definition] = variable.defs;
  if (definition.type !== "Variable") return null;
  const declarator = definition.node;
  if (
    !declarator?.init ||
    declarator.id.type !== "Identifier" ||
    declarator.id.name !== identifier.name ||
    declarator.parent?.type !== "VariableDeclaration" ||
    declarator.parent.kind !== "const"
  ) {
    return null;
  }
  if (unwrapExpression(declarator.init).type === "Identifier") return null;
  return declarator;
}

function isSameConst(identifier, declarator, context) {
  return identifier.type === "Identifier" && constDeclarator(identifier, context) === declarator;
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

function collectObserverSites(context, functionNode, declarator) {
  const observers = [];
  traverse(context, functionNode, (node) => {
    const matcher = rejectionMatcherCall(node, declarator, context);
    const handler =
      isRejectionHandlerCall(node, declarator, context) ||
      isImmediateObserver(node, declarator, context, isSameConst);
    if (handler) observers.push(node);
    else if (matcher) observers.push(matcher);
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
  const functionNode = findContainingFunction(declarator);
  if (!functionNode || functionNode !== findContainingFunction(matcher)) return false;
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
    return {
      MemberExpression(node) {
        const identifier = expectedIdentifier(node, context);
        if (!identifier) return;
        const declarator = constDeclarator(identifier, context);
        const call = matcherCall(node);
        if (
          !declarator ||
          !call ||
          !hasInterveningAwait(context, declarator, call, observerCache)
        ) {
          return;
        }
        context.report({ node, messageId: "delayedReject" });
      },
    };
  },
);
