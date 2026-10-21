"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
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
const TEST_CALLEES = new Set(["it", "test"]);

function isMessageMember(node) {
  const current = unwrap(node);
  return (
    current?.type === "MemberExpression" &&
    memberName(current) === "message" &&
    !isBodyMember(current.object)
  );
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

function calleeName(node) {
  while (node?.type === "ChainExpression") node = node.expression;
  if (node?.type === "SequenceExpression") return calleeName(node.expressions.at(-1));
  if (node?.type === "Identifier") return node.name;
  if (node?.type === "MemberExpression") {
    if (node.computed && node.property?.type === "Identifier") return null;
    if (memberName(node)) return calleeName(node.object);
  }
  if (node?.type === "CallExpression") return calleeName(node.callee);
  return null;
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
    schema: [],
    messages: { message: "Do not assert on err.message; check the error type or code instead." },
  },
  (context) => {
    if (isStaticAnalysisPath(context.filename) || isGeneratedTestPath(context.filename)) return {};
    const suppressed = ruleSuppression(context, "test-no-error-message-matching");
    let testDepth = 0;
    return {
      CallExpression(node) {
        if (TEST_CALLEES.has(calleeName(node.callee))) {
          testDepth += 1;
        }
        let callee = unwrap(node.callee);
        if (
          callee?.type === "MemberExpression" &&
          !callee.computed &&
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
          isMessageMember(expectArguments(matcherObject, context)?.[0]) &&
          !suppressed(node)
        ) {
          context.report({ node, messageId: "message" });
          return;
        }
        if (
          callee?.type === "MemberExpression" &&
          memberName(callee) === "includes" &&
          testDepth > 0 &&
          isMessageMember(callee.object) &&
          !suppressed(node)
        ) {
          context.report({ node, messageId: "message" });
        }
      },
      "CallExpression:exit"(node) {
        if (TEST_CALLEES.has(calleeName(node.callee))) testDepth -= 1;
      },
      BinaryExpression(node) {
        if (testDepth <= 0) return;
        if (!["!=", "==", "===", "!=="].includes(node.operator)) return;
        if (
          (isMessageMember(node.left) && node.right.type === "Literal") ||
          (isMessageMember(node.right) && node.left.type === "Literal")
        ) {
          if (!suppressed(node)) context.report({ node, messageId: "message" });
        }
      },
    };
  },
);

module.exports.__test = { propertyName };
