"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { literalString } = require("./module-mock-helpers");
const { EXPECT_MODULES, isExpectCallee, memberName, unwrap } = require("./test-message-expect");
const { localOrigin, moduleMember } = require("./test-no-wall-time-assertions-bindings");

const RULE_NAME = "test-no-style-class-assertion";
const TEST_FILE_PATTERN = /\.(?:test|spec)\.[cm]?[jt]sx?$/;
const EXPECT_MODIFIERS = new Set(["not", "resolves", "rejects"]);

function expectCallKind(node, context, seen = new Set()) {
  const target = unwrap(node);
  if (target?.type === "MemberExpression") {
    const name = memberName(target);
    if (name === "poll") {
      return expectCallKind(target.object, context, seen) ? "poll" : null;
    }
    if (name === "soft") {
      return expectCallKind(target.object, context, seen) ? "expect" : null;
    }
  }
  if (
    isExpectCallee(target, context, new Set(), EXPECT_MODULES, {
      allowForwarding: false,
      allowPoll: false,
    })
  ) {
    return "expect";
  }
  const member = moduleMember(target, context);
  if (member?.name === "expect" && EXPECT_MODULES.has(member.source)) {
    return "expect";
  }
  if (target?.type === "Identifier" && !seen.has(target.name)) {
    const origin = localOrigin(target, context);
    if (origin !== target) {
      seen.add(target.name);
      return expectCallKind(origin, context, seen);
    }
  }
  return null;
}

function pollReturnExpression(node) {
  const fn = unwrap(node);
  if (fn?.type !== "ArrowFunctionExpression" && fn?.type !== "FunctionExpression") return null;
  if (fn.body.type !== "BlockStatement") return fn.body;
  const [statement] = fn.body.body;
  return fn.body.body.length === 1 && statement.type === "ReturnStatement"
    ? statement.argument
    : null;
}

function isStyleClassExpression(node, context) {
  const expr = localOrigin(node, context);
  if (expr?.type === "MemberExpression") {
    return memberName(expr) === "className";
  }
  if (expr?.type !== "CallExpression") return false;
  const callee = unwrap(expr.callee);
  if (callee?.type !== "MemberExpression") return false;
  const method = memberName(callee);
  if (method === "contains") {
    const receiver = localOrigin(callee.object, context);
    return receiver?.type === "MemberExpression" && memberName(receiver) === "classList";
  }
  if (method === "getAttribute" && expr.arguments.length >= 1) {
    const attribute = literalString(localOrigin(expr.arguments[0], context));
    return attribute !== null && attribute.toLowerCase() === "class";
  }
  return false;
}

function isStyleClassAssertion(node, context) {
  const callee = unwrap(node.callee);
  if (callee?.type !== "MemberExpression") return false;
  let assertion = unwrap(callee.object);
  while (assertion?.type === "MemberExpression" && EXPECT_MODIFIERS.has(memberName(assertion))) {
    assertion = unwrap(assertion.object);
  }
  if (assertion?.type !== "CallExpression") return false;
  const kind = expectCallKind(assertion.callee, context);
  if (!kind) return false;
  if (memberName(callee) === "toHaveClass") return true;
  const actual =
    kind === "poll" ? pollReturnExpression(assertion.arguments[0]) : assertion.arguments[0];
  return isStyleClassExpression(actual, context);
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow asserting on styling class names in tests",
      recommended: false,
    },
    schema: [],
    messages: {
      styleClassAssertion:
        "Avoid asserting on styling class names. Assert the behavior instead: toBeVisible() / not.toBeVisible(), the accessible name or role, an aria-* state (aria-expanded, aria-selected, aria-current), or a data-* attribute the component sets for its state.",
    },
  },
  (context) => {
    if (!TEST_FILE_PATTERN.test(context.filename.replace(/\\/g, "/"))) return {};
    const suppressed = ruleSuppression(context, RULE_NAME, { requireReason: true });
    return {
      CallExpression(node) {
        if (isStyleClassAssertion(node, context) && !suppressed(node)) {
          context.report({ node, messageId: "styleClassAssertion" });
        }
      },
    };
  },
);
