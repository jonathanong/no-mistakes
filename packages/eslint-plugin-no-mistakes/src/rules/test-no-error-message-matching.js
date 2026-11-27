"use strict";

const { rule } = require("../helpers");
const { isUnresolved, moduleLoad } = require("./module-load");
const { reportUnanalyzable, UNANALYZABLE_SCHEMA } = require("./rule-unanalyzable");
const { ruleSuppression } = require("../rule-suppression");
const { isTestRunner } = require("./test-message-runner");
const { capturedMessageOrigin } = require("./test-message-captured-value");
const {
  expectArguments,
  isExpectCall,
  memberName,
  propertyName,
  unwrap,
} = require("./test-message-expect");

const MATCHERS = new Set([
  "contain",
  "contains",
  "equal",
  "equals",
  "include",
  "includes",
  "match",
  "toBe",
  "toEqual",
  "toStrictEqual",
  "toContain",
  "toMatch",
  "toMatchInlineSnapshot",
  "toMatchSnapshot",
]);
const MODIFIERS = new Set(
  "and at be been deep has have is not of rejects resolves same that to which with".split(" "),
);

function isMessageMember(node) {
  const current = unwrap(node);
  return (
    current?.type === "MemberExpression" &&
    memberName(current) === "message" &&
    !isBodyMember(current.object)
  );
}

function isMessageValue(node, context) {
  return isMessageMember(node) || isMessageMember(capturedMessageOrigin(node, context));
}

function isBodyMember(node) {
  const current = unwrap(node);
  return current?.type === "MemberExpression" && memberName(current) === "body";
}

function isStaticAnalysisPath(filename) {
  const normalized = filename.replace(/\\/g, "/");
  return (
    normalized.startsWith("static-code-analysis/") || normalized.includes("/static-code-analysis/")
  );
}

function isGeneratedTestPath(filename) {
  return filename.replace(/\\/g, "/").includes(".generated.");
}

function forwardedArguments(node) {
  const current = unwrap(node);
  if (current?.type !== "CallExpression") return null;
  const callee = unwrap(current.callee);
  const args = current.arguments;
  if (callee?.type !== "MemberExpression") return args;
  const name = memberName(callee);
  if (name === "call") return args.slice(1);
  if (name === "apply" && unwrap(args[1])?.type === "ArrayExpression") {
    return unwrap(args[1]).elements;
  }
  return args;
}

function opaqueExpect(node, context) {
  const current = unwrap(node);
  if (current?.type !== "CallExpression") return false;
  const callee = unwrap(current.callee);
  const target =
    callee?.type === "MemberExpression" && memberName(callee) === "expect" ? callee.object : callee;
  return isUnresolved(moduleLoad(target, context));
}

function unwrapMatcherObject(node) {
  let current = unwrap(node);
  while (current?.type === "MemberExpression") {
    const name = memberName(current);
    if (!MODIFIERS.has(name)) return null;
    current = unwrap(current.object);
  }
  return current;
}

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow asserting on error message strings", recommended: false },
    schema: [{ type: "object", properties: { unanalyzable: UNANALYZABLE_SCHEMA } }],
    messages: {
      message: "Do not assert on err.message; check the error type or code instead.",
      unanalyzable:
        "Cannot tell whether {{subject}} asserts on an error message. Use a static expect import.",
    },
  },
  (context) => {
    if (isStaticAnalysisPath(context.filename) || isGeneratedTestPath(context.filename)) return {};
    const suppressed = ruleSuppression(context, "test-no-error-message-matching");
    let testDepth = 0;
    return {
      CallExpression(node) {
        if (isTestRunner(node.callee, context)) {
          testDepth += 1;
        }
        let callee = unwrap(node.callee);
        if (
          callee?.type === "MemberExpression" &&
          (memberName(callee) === "call" || memberName(callee) === "apply")
        ) {
          callee = unwrap(callee.object);
        }
        const matcherObject =
          callee?.type === "MemberExpression" ? unwrapMatcherObject(callee.object) : null;
        if (
          callee?.type === "MemberExpression" &&
          MATCHERS.has(memberName(callee)) &&
          isExpectCall(matcherObject, context) &&
          isMessageValue(expectArguments(matcherObject, context)?.[0], context) &&
          !suppressed(node)
        ) {
          context.report({ node, messageId: "message" });
          return;
        }
        if (
          callee?.type === "MemberExpression" &&
          MATCHERS.has(memberName(callee)) &&
          !isExpectCall(matcherObject, context) &&
          opaqueExpect(matcherObject, context) &&
          isMessageValue(forwardedArguments(matcherObject)?.[0], context) &&
          !suppressed(node)
        ) {
          reportUnanalyzable(context, node, "this assertion");
        }
        if (
          callee?.type === "MemberExpression" &&
          memberName(callee) === "includes" &&
          testDepth > 0 &&
          isMessageValue(callee.object, context) &&
          !suppressed(node)
        ) {
          context.report({ node, messageId: "message" });
        }
      },
      "CallExpression:exit"(node) {
        if (isTestRunner(node.callee, context)) testDepth -= 1;
      },
      BinaryExpression(node) {
        if (testDepth <= 0) return;
        if (!["!=", "==", "===", "!=="].includes(node.operator)) return;
        const right = unwrap(node.right);
        const left = unwrap(node.left);
        if (
          (isMessageValue(node.left, context) && right?.type === "Literal") ||
          (isMessageValue(node.right, context) && left?.type === "Literal")
        ) {
          if (!suppressed(node)) context.report({ node, messageId: "message" });
        }
      },
    };
  },
);

module.exports.__test = { propertyName };
