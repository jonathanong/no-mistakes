"use strict";

const { rule } = require("../helpers");
const { findContainingFunction, traverse, unwrapExpression } = require("./async-ast");
const { canReachMatcher, contains, executesBefore } = require("./test-no-delayed-rejects-flow");
const { isNonRejectingHandler } = require("./test-no-delayed-rejects-handlers");

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

function literalPropertyName(member) {
  if (!member.computed && member.property.type === "Identifier") return member.property.name;
  if (member.computed && member.property.type === "Literal") return member.property.value;
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
  const callee = node.callee;
  const object = unwrapExpression(callee.object);
  if (!isSameConst(object, declarator, context)) return false;
  const property = literalPropertyName(callee);
  if (property === "catch")
    return node.arguments.length >= 1 && isNonRejectingHandler(node.arguments[0]);
  return (
    property === "then" && node.arguments.length >= 2 && isNonRejectingHandler(node.arguments[1])
  );
}

function hasObserverBeforeSuspension(context, functionNode, declarator, suspension) {
  let found = false;
  traverse(context, functionNode, (node) => {
    if (isRejectionHandlerCall(node, declarator, context) && executesBefore(node, suspension)) {
      found = true;
    }
  });
  return found;
}

function hasInterveningAwait(context, declarator, matcher) {
  const functionNode = findContainingFunction(declarator);
  if (!functionNode || functionNode !== findContainingFunction(matcher)) return false;

  let found = false;
  traverse(context, functionNode, (node) => {
    if (
      (node.type === "AwaitExpression" || (node.type === "ForOfStatement" && node.await)) &&
      node.range[0] >= declarator.init.range[1] &&
      node.range[0] < matcher.range[1] &&
      !contains(node, matcher) &&
      canReachMatcher(node, matcher, functionNode) &&
      !hasObserverBeforeSuspension(context, functionNode, declarator, node)
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
  (context) => ({
    MemberExpression(node) {
      const identifier = expectedIdentifier(node, context);
      if (!identifier) return;
      const declarator = constDeclarator(identifier, context);
      const call = matcherCall(node);
      if (!declarator || !call || !hasInterveningAwait(context, declarator, call)) return;
      context.report({ node, messageId: "delayedReject" });
    },
  }),
);
