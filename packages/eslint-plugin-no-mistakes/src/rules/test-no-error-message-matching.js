"use strict";

const { rule } = require("../helpers");

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

function propertyName(node) {
  if (node.type === "Identifier") return node.name;
  return node.type === "Literal" ? String(node.value) : null;
}

function unwrap(node) {
  let current = node;
  while (
    current?.type === "ChainExpression" ||
    current?.type === "TSAsExpression" ||
    current?.type === "TSTypeAssertion"
  ) {
    current = current.expression;
  }
  return current;
}

function isMessageMember(node) {
  const current = unwrap(node);
  return current?.type === "MemberExpression" && propertyName(current.property) === "message";
}

function isExpectCall(node) {
  const current = unwrap(node);
  if (current?.type !== "CallExpression") return false;
  if (current.callee.type === "Identifier") return current.callee.name === "expect";
  return (
    current.callee.type === "MemberExpression" &&
    ["soft", "poll"].includes(propertyName(current.callee.property)) &&
    current.callee.object?.type === "Identifier" &&
    current.callee.object.name === "expect"
  );
}

function unwrapMatcherObject(node) {
  let current = unwrap(node);
  while (current?.type === "MemberExpression") {
    const name = propertyName(current.property);
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
  (context) => ({
    CallExpression(node) {
      const matcherObject =
        node.callee.type === "MemberExpression" ? unwrapMatcherObject(node.callee.object) : null;
      if (
        node.callee.type === "MemberExpression" &&
        MATCHERS.has(propertyName(node.callee.property)) &&
        isExpectCall(matcherObject) &&
        isMessageMember(matcherObject.arguments?.[0])
      ) {
        context.report({ node, messageId: "message" });
        return;
      }
      if (
        node.callee.type === "MemberExpression" &&
        propertyName(node.callee.property) === "includes" &&
        isMessageMember(node.callee.object)
      ) {
        context.report({ node, messageId: "message" });
      }
    },
    BinaryExpression(node) {
      if (!["==", "===", "!=="].includes(node.operator)) return;
      if (
        (isMessageMember(node.left) && node.right.type === "Literal") ||
        (isMessageMember(node.right) && node.left.type === "Literal")
      ) {
        context.report({ node, messageId: "message" });
      }
    },
  }),
);
