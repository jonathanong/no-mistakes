"use strict";

const { rule } = require("../helpers");

function isReactUse(callee) {
  return (
    callee?.type === "MemberExpression" &&
    callee.object?.type === "Identifier" &&
    callee.object.name === "React" &&
    callee.property?.type === "Identifier" &&
    callee.property.name === "use"
  );
}

function isPromiseResolve(node) {
  return (
    node?.type === "CallExpression" &&
    node.callee?.type === "MemberExpression" &&
    node.callee.object?.type === "Identifier" &&
    node.callee.object.name === "Promise" &&
    node.callee.property?.type === "Identifier" &&
    node.callee.property.name === "resolve"
  );
}

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow React.use(Promise.resolve(...))", recommended: false },
    schema: [],
    messages: {
      resolve: "Avoid React.use(Promise.resolve(...)); pass the promise directly or use await.",
    },
  },
  (context) => ({
    CallExpression(node) {
      if (!isReactUse(node.callee) || !isPromiseResolve(node.arguments[0])) return;
      context.report({ node, messageId: "resolve" });
    },
  }),
);
